mod block;
mod header;
mod helpers;

pub(crate) use header::*;
pub(crate) use helpers::*;

use magnus::{
    function,
    prelude::*,
    value::{InnerValue, Lazy},
    Error, ExceptionClass, RModule, Ruby,
};

static MODULE_ROOT: Lazy<RModule> = Lazy::new(|ruby| ruby.define_module("Lz4Flex").unwrap());

pub(crate) fn base_error_class() -> ExceptionClass {
    static BASE_ERROR_CLASS: Lazy<ExceptionClass> = Lazy::new(|ruby| {
        MODULE_ROOT
            .get_inner_with(ruby)
            .define_error("Error", ruby.exception_standard_error())
            .unwrap()
    });
    unsafe { BASE_ERROR_CLASS.get_inner_with(&Ruby::get_unchecked()) }
}

pub(crate) fn encode_error_class() -> ExceptionClass {
    static ENCODE_ERROR_CLASS: Lazy<ExceptionClass> = Lazy::new(|ruby| {
        MODULE_ROOT
            .get_inner_with(ruby)
            .define_error("EncodeError", base_error_class())
            .unwrap()
    });
    unsafe { ENCODE_ERROR_CLASS.get_inner_with(&Ruby::get_unchecked()) }
}

pub(crate) fn decode_error_class() -> ExceptionClass {
    static DECODE_ERROR_CLASS: Lazy<ExceptionClass> = Lazy::new(|ruby| {
        MODULE_ROOT
            .get_inner_with(ruby)
            .define_error("DecodeError", base_error_class())
            .unwrap()
    });
    unsafe { DECODE_ERROR_CLASS.get_inner_with(&Ruby::get_unchecked()) }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = MODULE_ROOT.get_inner_with(ruby);
    let _ = decode_error_class();
    let _ = encode_error_class();

    module.define_singleton_method("compress", function!(block::compress, 1))?;
    module.define_singleton_method("decompress", function!(block::decompress, 1))?;
    module
        .singleton_class()?
        .define_alias("deflate", "compress")?;
    module
        .singleton_class()?
        .define_alias("inflate", "decompress")?;

    let varint_module = module.define_module("VarInt")?;
    varint_module.define_singleton_method("compress", function!(block::compress_varint, 1))?;
    varint_module.define_singleton_method("decompress", function!(block::decompress_varint, 1))?;

    Ok(())
}
