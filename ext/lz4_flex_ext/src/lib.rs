mod block;
mod header;

pub(crate) use header::*;

use boltffi::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub(crate) enum Encoding {
    Utf8 = 0,
    Binary = 1,
    UsAscii = 2,
}

impl Encoding {
    pub(crate) const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Utf8),
            1 => Some(Self::Binary),
            2 => Some(Self::UsAscii),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Lz4FlexError {
    Base,
    Encode,
    Decode,
}

impl Lz4FlexError {
    pub(crate) fn base(_message: impl Into<String>) -> Self {
        Self::Base
    }

    pub(crate) fn encode(_message: impl Into<String>) -> Self {
        Self::Encode
    }

    pub(crate) fn decode(_message: impl Into<String>) -> Self {
        Self::Decode
    }
}

#[export]
pub fn compress(input: &[u8], encoding_val: u8) -> Vec<u8> {
    block::compress(input, encoding_val).unwrap_or_default()
}

#[export]
pub fn decompress(input: &[u8]) -> Vec<u8> {
    block::decompress(input).unwrap_or_default()
}

#[export]
pub fn max_compressed_size(input_len: u32) -> u32 {
    block::max_compressed_size(input_len)
}

#[export]
pub fn compress_into(input: &[u8], encoding_val: u8, output: &mut [u8]) -> u32 {
    block::compress_into(input, encoding_val, output).unwrap_or(u32::MAX)
}

#[export]
pub fn decompress_into(input: &[u8], output: &mut [u8]) -> u32 {
    block::decompress_into(input, output).unwrap_or(u32::MAX)
}

#[export]
pub fn decompress_payload_into(
    input: &[u8],
    data_offset: u32,
    expected_size: u32,
    output: &mut [u8],
) -> u32 {
    block::decompress_payload_into(input, data_offset, expected_size, output).unwrap_or(u32::MAX)
}

#[export]
pub fn get_compressed_encoding(input: &[u8]) -> u8 {
    block::get_compressed_encoding(input).unwrap_or(255)
}

#[export]
pub fn get_decompressed_size(input: &[u8]) -> u32 {
    block::get_decompressed_size(input).unwrap_or(u32::MAX)
}

#[export]
pub fn get_decompression_metadata(input: &[u8]) -> u64 {
    block::get_decompression_metadata(input).unwrap_or(u64::MAX)
}

pub struct VarInt;

#[export]
impl VarInt {
    pub fn compress(input: &[u8]) -> Vec<u8> {
        block::compress_varint(input).unwrap_or_default()
    }

    pub fn decompress(input: &[u8]) -> Vec<u8> {
        block::decompress_varint(input).unwrap_or_default()
    }
}
