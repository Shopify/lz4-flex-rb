mod helpers;

pub use helpers::*;

use lz4_flex::block::{compress_into, decompress_into, get_maximum_output_size, uncompressed_size};

use magnus::{
    exception::standard_error,
    function,
    prelude::*,
    value::{InnerValue, Lazy},
    Error, ExceptionClass, RModule, RString, Ruby,
};

/// Encodes a block of data using LZ4 compression, without encoding awareness of the string.
fn compress_block(ruby: &Ruby, input: LockedRString) -> Result<RString, Error> {
    let bufsize = get_maximum_output_size(input.len()) + 4; // +4 for prepended size
    let mut output = RStringMut::buf_new(bufsize);
    output.resize(bufsize);

    let outbuf = output.as_mut_slice();

    // Write size to first 4 bytes
    outbuf[..4].copy_from_slice(&(input.len() as u32).to_le_bytes());
    let outbuf = &mut outbuf[4..];

    let outsize = nogvl_if_large(output.len(), || compress_into(input.as_slice(), outbuf))
        .map_err(|e| Error::new(ENCODE_ERROR_CLASS.get_inner_with(ruby), e.to_string()))?;

    output.resize(outsize + 4);

    Ok(output.into_inner())
}

/// Decodes a block of data using LZ4 compression, without encoding awareness of the string.
fn decompress_block(ruby: &Ruby, input: RString) -> Result<RString, Error> {
    let input_slice = unsafe { input.as_slice() };
    let (size, input_slice) = uncompressed_size(input_slice)
        .map_err(|e| Error::new(DECODE_ERROR_CLASS.get_inner_with(ruby), e.to_string()))?;

    let mut output = RStringMut::buf_new(size);
    output.resize(size);

    nogvl_if_large(output.len(), || {
        decompress_into(input_slice, output.as_mut_slice())
    })
    .map_err(|e| Error::new(DECODE_ERROR_CLASS.get_inner_with(ruby), e.to_string()))?;

    Ok(output.into_inner())
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
    module.define_singleton_method("compress_block", function!(compress_block, 1))?;
    module.define_singleton_method("decompress_block", function!(decompress_block, 1))?;

    Ok(())
}
