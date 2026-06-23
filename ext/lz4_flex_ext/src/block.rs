use lz4_flex::block::{
    compress_into as lz4_compress_into, decompress_into as lz4_decompress_into,
    get_maximum_output_size,
};

use crate::{Encoding, Header, Lz4FlexError};

static_assertions::assert_eq_size!(Header, [u8; 8]);

fn compress_with_header(
    header: &Header,
    input: &[u8],
    varint: bool,
) -> Result<Vec<u8>, Lz4FlexError> {
    let bufsize = get_maximum_output_size(input.len()) + Header::MAX_SERIALIZED_SIZE;
    let mut output = vec![0; bufsize];

    let written = compress_with_header_into(header, input, varint, &mut output)?;
    output.truncate(written as usize);
    Ok(output)
}

pub(crate) fn max_compressed_size(input_len: u32) -> u32 {
    get_maximum_output_size(input_len as usize)
        .saturating_add(Header::MAX_SERIALIZED_SIZE)
        .try_into()
        .unwrap_or(u32::MAX)
}

fn compress_with_header_into(
    header: &Header,
    input: &[u8],
    varint: bool,
    output: &mut [u8],
) -> Result<u32, Lz4FlexError> {
    let (header_len, outbuf) = if varint {
        header.write_varint_to(output)
    } else {
        header.write_to(output)
    };

    let outsize =
        lz4_compress_into(input, outbuf).map_err(|e| Lz4FlexError::encode(e.to_string()))?;
    (outsize + header_len)
        .try_into()
        .map_err(|_| Lz4FlexError::encode("compressed output too large"))
}

pub(crate) fn compress(input: &[u8], encoding_val: u8) -> Result<Vec<u8>, Lz4FlexError> {
    let encoding = Encoding::from_u8(encoding_val)
        .ok_or_else(|| Lz4FlexError::base("unsupported encoding"))?;
    let header = Header::new(input.len() as u32, encoding);

    compress_with_header(&header, input, false)
}

pub(crate) fn compress_into(
    input: &[u8],
    encoding_val: u8,
    output: &mut [u8],
) -> Result<u32, Lz4FlexError> {
    let encoding = Encoding::from_u8(encoding_val)
        .ok_or_else(|| Lz4FlexError::base("unsupported encoding"))?;
    let header = Header::new(input.len() as u32, encoding);

    compress_with_header_into(&header, input, false, output)
}

pub(crate) fn compress_varint(input: &[u8]) -> Result<Vec<u8>, Lz4FlexError> {
    let header = Header::new(input.len() as u32, Encoding::Binary);

    compress_with_header(&header, input, true)
}

fn decompress_with_header(header: &Header, input_slice: &[u8]) -> Result<Vec<u8>, Lz4FlexError> {
    let mut output = vec![0; header.size as usize];

    decompress_with_header_into(header, input_slice, &mut output)?;

    Ok(output)
}

fn decompress_with_header_into(
    header: &Header,
    input_slice: &[u8],
    output: &mut [u8],
) -> Result<u32, Lz4FlexError> {
    decompress_size_into(header.size, input_slice, output)
}

fn decompress_size_into(
    expected_size: u32,
    input_slice: &[u8],
    output: &mut [u8],
) -> Result<u32, Lz4FlexError> {
    if output.len() < expected_size as usize {
        return Err(Lz4FlexError::decode("output buffer too small"));
    }

    lz4_decompress_into(input_slice, &mut output[..expected_size as usize])
        .map_err(|e| Lz4FlexError::decode(e.to_string()))?;

    Ok(expected_size)
}

pub(crate) fn decompress(input: &[u8]) -> Result<Vec<u8>, Lz4FlexError> {
    let (header, input_slice) = Header::from_bytes(input)?;

    decompress_with_header(&header, input_slice)
}

pub(crate) fn decompress_into(input: &[u8], output: &mut [u8]) -> Result<u32, Lz4FlexError> {
    let (header, input_slice) = Header::from_bytes(input)?;

    decompress_with_header_into(&header, input_slice, output)
}

pub(crate) fn decompress_payload_into(
    input: &[u8],
    data_offset: u32,
    expected_size: u32,
    output: &mut [u8],
) -> Result<u32, Lz4FlexError> {
    let data_offset = data_offset as usize;
    if data_offset > input.len() {
        return Err(Lz4FlexError::decode("invalid data offset"));
    }

    decompress_size_into(expected_size, &input[data_offset..], output)
}

pub(crate) fn get_compressed_encoding(input: &[u8]) -> Result<u8, Lz4FlexError> {
    let (header, _) = Header::from_bytes(input)?;

    Ok(header.encoding()? as u8)
}

pub(crate) fn get_decompressed_size(input: &[u8]) -> Result<u32, Lz4FlexError> {
    let (header, _) = Header::from_bytes(input)?;

    Ok(header.size)
}

pub(crate) fn get_decompression_metadata(input: &[u8]) -> Result<u64, Lz4FlexError> {
    let (header, rest) = Header::from_bytes(input)?;
    let encoding = header.encoding()? as u64;
    let data_offset = (input.len() - rest.len()) as u64;

    Ok((encoding << 56) | (data_offset << 32) | u64::from(header.size))
}

pub(crate) fn decompress_varint(input: &[u8]) -> Result<Vec<u8>, Lz4FlexError> {
    let (header, input_slice) = Header::from_varint(input)?;

    decompress_with_header(&header, input_slice)
}
