use lz4_flex::block::{compress_into, decompress_into, get_maximum_output_size};

use crate::{Encoding, Header, Lz4FlexError};

static_assertions::assert_eq_size!(Header, [u8; 8]);

fn compress_with_header(
    header: &Header,
    input: &[u8],
    varint: bool,
) -> Result<Vec<u8>, Lz4FlexError> {
    let bufsize = get_maximum_output_size(input.len()) + Header::MAX_SERIALIZED_SIZE;
    let mut output = vec![0; bufsize];

    let (header_len, outbuf) = if varint {
        header.write_varint_to(&mut output)
    } else {
        header.write_to(&mut output)
    };

    let outsize = compress_into(input, outbuf).map_err(|e| Lz4FlexError::encode(e.to_string()))?;
    output.truncate(outsize + header_len);
    Ok(output)
}

pub(crate) fn compress(input: &[u8], encoding_val: u8) -> Result<Vec<u8>, Lz4FlexError> {
    let encoding = Encoding::from_u8(encoding_val)
        .ok_or_else(|| Lz4FlexError::base("unsupported encoding"))?;
    let header = Header::new(input.len() as u32, encoding);

    compress_with_header(&header, input, false)
}

pub(crate) fn compress_varint(input: &[u8]) -> Result<Vec<u8>, Lz4FlexError> {
    let header = Header::new(input.len() as u32, Encoding::Binary);

    compress_with_header(&header, input, true)
}

fn decompress_with_header(header: &Header, input_slice: &[u8]) -> Result<Vec<u8>, Lz4FlexError> {
    let mut output = vec![0; header.size as usize];

    decompress_into(input_slice, &mut output).map_err(|e| Lz4FlexError::decode(e.to_string()))?;

    Ok(output)
}

pub(crate) fn decompress(input: &[u8]) -> Result<Vec<u8>, Lz4FlexError> {
    let (header, input_slice) = Header::from_bytes(input)?;

    decompress_with_header(&header, input_slice)
}

pub(crate) fn get_compressed_encoding(input: &[u8]) -> Result<u8, Lz4FlexError> {
    let (header, _) = Header::from_bytes(input)?;

    Ok(header.encoding()? as u8)
}

pub(crate) fn get_decompressed_size(input: &[u8]) -> Result<u32, Lz4FlexError> {
    let (header, _) = Header::from_bytes(input)?;

    Ok(header.size)
}

pub(crate) fn decompress_varint(input: &[u8]) -> Result<Vec<u8>, Lz4FlexError> {
    let (header, input_slice) = Header::from_varint(input)?;

    decompress_with_header(&header, input_slice)
}
