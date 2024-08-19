use lz4_flex::block::{compress_into, decompress_into, get_maximum_output_size};

use crate::Header;
use crate::LockedRString;
use crate::{decode_error_class, encode_error_class, helpers::*};

use magnus::{prelude::*, Error, RString};

static_assertions::assert_eq_size!(Header, [u8; 8]);

pub(crate) fn compress_with_header(
    header: &Header,
    input: LockedRString,
    varint: bool,
) -> Result<RString, Error> {
    let bufsize = get_maximum_output_size(input.len()) + Header::MAX_SERIALIZED_SIZE;
    let mut output = RStringMut::buf_new(bufsize);
    output.expand(bufsize);

    let outbuf = output.as_mut_slice();

    let (header_len, outbuf) = if varint {
        header.write_varint_to(outbuf)
    } else {
        header.write_to(outbuf)
    };

    let outsize = nogvl_if_large(output.capacity(), || {
        compress_into(input.as_slice(), outbuf)
    })
    .map_err(|e| Error::new(encode_error_class(), e.to_string()))?;

    output.set_len(outsize + header_len);
    Ok(output.into_inner())
}

/// Encodes a block of data using LZ4 compression, with encoding awareness of the string.
pub(crate) fn compress(input: LockedRString) -> Result<RString, Error> {
    let header = Header::new(input.len() as u32, input.encoding()?);

    compress_with_header(&header, input, false)
}

pub(crate) fn compress_varint(input: LockedRString) -> Result<RString, Error> {
    let header = Header::new(input.len() as u32, Encoding::Binary);

    compress_with_header(&header, input, true)
}

pub(crate) fn decompress_with_header(
    header: &Header,
    input_slice: &[u8],
) -> Result<RString, Error> {
    let mut output = RStringMut::buf_new(header.size as usize);
    output.expand(header.size as usize);

    nogvl_if_large(output.capacity(), || {
        decompress_into(input_slice, output.as_mut_slice())
    })
    .map_err(|e| Error::new(decode_error_class(), e.to_string()))?;

    output.set_len(header.size as usize);
    let output = output.into_inner();
    output.enc_set(header.encoding().encindex())?;

    Ok(output)
}

pub(crate) fn decompress(input: LockedRString) -> Result<RString, Error> {
    let input_slice = input.as_slice();
    let (header, input_slice) = Header::from_bytes(input_slice)?;

    decompress_with_header(&header, input_slice)
}

pub(crate) fn decompress_varint(input: LockedRString) -> Result<RString, Error> {
    let input_slice = input.as_slice();
    let (header, input_slice) = Header::from_varint(input_slice)?;

    decompress_with_header(&header, input_slice)
}
