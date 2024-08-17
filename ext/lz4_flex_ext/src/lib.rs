mod header;
mod helpers;

pub(crate) use header::*;
pub(crate) use helpers::*;

use lz4_flex::block::{compress_into, decompress_into, get_maximum_output_size};

use magnus::{
    exception::standard_error,
    function,
    prelude::*,
    value::{InnerValue, Lazy},
    Error, ExceptionClass, RModule, RString, Ruby,
};
use rb_sys::ruby_abi_version;

static_assertions::assert_eq_size!(Header, [u8; 8]);

/// Encodes a block of data using LZ4 compression, with encoding awareness of the string.
fn compress(input: LockedRString) -> Result<RString, Error> {
    let input_len = input.len();
    let encoding = input.encoding()?;
    let bufsize = get_maximum_output_size(input.len()) + size_of::<Header>(); // +8 for prepended size
    let mut output = RStringMut::buf_new(bufsize);
    output.expand(bufsize);

    let outbuf = output.as_mut_slice();
    let header = Header::new(input_len as u32, encoding);
    let outbuf = header.write_to(outbuf);

    let outsize = nogvl_if_large(output.capacity(), || {
        compress_into(input.as_slice(), outbuf)
    })
    .map_err(|e| Error::new(encode_error_class(), e.to_string()))?;

    output.set_len(outsize + size_of::<Header>());
    Ok(output.into_inner())
}

fn decompress(input: LockedRString) -> Result<RString, Error> {
    let input_slice = input.as_slice();
    let (header, input_slice) = Header::from_bytes(input_slice)?;
    let mut output = RStringMut::buf_new(header.size as usize);
    output.expand(header.size as usize);

    nogvl_if_large(output.capacity(), || {
        decompress_into(input_slice, output.as_mut_slice())
    })
    .map_err(|e| Error::new(decode_error_class(), e.to_string()))?;

    output.set_len(header.size as usize);
    let output = output.into_inner();
    output.enc_set(header.encoding.encindex())?;

    Ok(output)
}

static MODULE_ROOT: Lazy<RModule> = Lazy::new(|ruby| ruby.define_module("Lz4Flex").unwrap());

fn base_error_class() -> ExceptionClass {
    static BASE_ERROR_CLASS: Lazy<ExceptionClass> = Lazy::new(|ruby| {
        MODULE_ROOT
            .get_inner_with(ruby)
            .define_error("Error", standard_error())
            .unwrap()
    });
    unsafe { BASE_ERROR_CLASS.get_inner_with(&Ruby::get_unchecked()) }
}

fn encode_error_class() -> ExceptionClass {
    static ENCODE_ERROR_CLASS: Lazy<ExceptionClass> = Lazy::new(|ruby| {
        MODULE_ROOT
            .get_inner_with(ruby)
            .define_error("EncodeError", base_error_class())
            .unwrap()
    });
    unsafe { ENCODE_ERROR_CLASS.get_inner_with(&Ruby::get_unchecked()) }
}

fn decode_error_class() -> ExceptionClass {
    static DECODE_ERROR_CLASS: Lazy<ExceptionClass> = Lazy::new(|ruby| {
        MODULE_ROOT
            .get_inner_with(ruby)
            .define_error("DecodeError", base_error_class())
            .unwrap()
    });
    unsafe { DECODE_ERROR_CLASS.get_inner_with(&Ruby::get_unchecked()) }
}

ruby_abi_version!();

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = MODULE_ROOT.get_inner_with(ruby);
    let _ = decode_error_class();
    let _ = encode_error_class();

    module.define_singleton_method("compress", function!(compress, 1))?;
    module.define_singleton_method("decompress", function!(decompress, 1))?;

    Ok(())
}
