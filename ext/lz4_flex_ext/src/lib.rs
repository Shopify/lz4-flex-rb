mod helpers;

pub use helpers::*;

use lz4_flex::block::{compress_into, decompress_into, get_maximum_output_size};

use magnus::{
    exception::standard_error,
    function,
    prelude::*,
    value::{InnerValue, Lazy},
    Error, ExceptionClass, RModule, RString, Ruby,
};

#[repr(C)]
struct Header {
    version: u8,
    encoding: Encoding, // u8
    __reserved: [u8; 2],
    size: u32,
}

static_assertions::assert_eq_size!(Header, [u8; 8]);

impl Header {
    fn new(size: u32, encoding: Encoding) -> Self {
        Self {
            version: 1,
            encoding,
            __reserved: [0; 2],
            size,
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let version = bytes[0];
        let encoding = Encoding::from_u8(bytes[1])?;
        let __reserved = [bytes[2], bytes[3]];
        let size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

        if version != 1 {
            return Err(Error::new(
                BASE_ERROR_CLASS.get_inner_with(&unsafe { Ruby::get_unchecked() }),
                "invalid version number in header",
            ));
        }

        Ok((
            Self {
                version,
                encoding,
                __reserved,
                size,
            },
            &bytes[8..],
        ))
    }

    fn write_to<'a>(&self, buf: &'a mut [u8]) -> &'a mut [u8] {
        let header_section = &mut buf[0..8];
        header_section[0] = self.version;
        header_section[1] = self.encoding as u8;
        header_section[2] = self.__reserved[0];
        header_section[3] = self.__reserved[1];
        header_section[4..8].copy_from_slice(&self.size.to_le_bytes());
        &mut buf[8..]
    }
}

/// Encodes a block of data using LZ4 compression, with encoding awareness of the string.
fn compress(ruby: &Ruby, input: LockedRString) -> Result<RString, Error> {
    let input_len = input.len();
    let encoding = input.encoding()?;
    let bufsize = get_maximum_output_size(input.len()) + size_of::<Header>(); // +8 for prepended size
    let mut output = RStringMut::buf_new(bufsize);
    output.resize(bufsize);

    let outbuf = output.as_mut_slice();
    let header = Header::new(input_len as u32, encoding);
    let outbuf = header.write_to(outbuf);

    let outsize = nogvl_if_large(output.len(), || compress_into(input.as_slice(), outbuf))
        .map_err(|e| Error::new(ENCODE_ERROR_CLASS.get_inner_with(ruby), e.to_string()))?;

    output.resize(outsize + 8);
    Ok(output.into_inner())
}

fn decompress(ruby: &Ruby, input: RString) -> Result<RString, Error> {
    let input_slice = unsafe { input.as_slice() };
    let (header, input_slice) = Header::from_bytes(input_slice)?;
    let mut output = RStringMut::buf_new(header.size as usize);
    output.resize(header.size as usize);

    nogvl_if_large(output.len(), || {
        decompress_into(input_slice, output.as_mut_slice())
    })
    .map_err(|e| Error::new(DECODE_ERROR_CLASS.get_inner_with(ruby), e.to_string()))?;

    let output = output.into_inner();
    output.enc_set(header.encoding.encindex())?;

    Ok(output)
}

static MODULE_ROOT: Lazy<RModule> = Lazy::new(|ruby| ruby.define_module("Lz4Flex").unwrap());

static BASE_ERROR_CLASS: Lazy<ExceptionClass> = Lazy::new(|ruby| {
    MODULE_ROOT
        .get_inner_with(ruby)
        .define_error("Error", standard_error())
        .unwrap()
});

static ENCODE_ERROR_CLASS: Lazy<ExceptionClass> = Lazy::new(|ruby| {
    MODULE_ROOT
        .get_inner_with(ruby)
        .define_error("EncodeError", BASE_ERROR_CLASS.get_inner_with(ruby))
        .unwrap()
});

static DECODE_ERROR_CLASS: Lazy<ExceptionClass> = Lazy::new(|ruby| {
    MODULE_ROOT
        .get_inner_with(ruby)
        .define_error("DecodeError", BASE_ERROR_CLASS.get_inner_with(ruby))
        .unwrap()
});

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = MODULE_ROOT.get_inner_with(ruby);
    module.define_singleton_method("compress", function!(compress, 1))?;
    module.define_singleton_method("decompress", function!(decompress, 1))?;

    Ok(())
}
