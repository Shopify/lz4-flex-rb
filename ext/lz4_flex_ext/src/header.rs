use magnus::Error;
use serde::{Deserialize, Serialize};

use crate::{base_error_class, decode_error_class, Encoding};

#[derive(Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub(crate) struct VersionAndEncoding(u8);

impl VersionAndEncoding {
    const fn new(version: u8, encoding: Encoding) -> Self {
        Self((version << 4) | encoding as u8)
    }

    const fn version(&self) -> u8 {
        self.0 >> 4
    }

    const fn encoding(&self) -> Encoding {
        Encoding::from_u8(self.0 & 0b1111)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub(crate) struct Header {
    pub(crate) metadata: VersionAndEncoding,
    pub(crate) size: u32,
}

impl Header {
    pub(crate) const MAX_SERIALIZED_SIZE: usize = size_of::<Self>();

    pub(crate) fn from_varint(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let mut val = 0u32;
        let mut varbyte_len = 0;

        for &b in bytes.iter() {
            val |= ((b & 0x7F) as u32) << (7 * varbyte_len);
            varbyte_len += 1;

            if b & 0x80 == 0 {
                let header = Self {
                    metadata: VersionAndEncoding::new(0, Encoding::Binary),
                    size: val,
                };

                return Ok((header, &bytes[varbyte_len..]));
            }

            if varbyte_len > 5 {
                return Err(Error::new(decode_error_class(), "varint too long"));
            }
        }

        Err(Error::new(decode_error_class(), "unexpected end of varint"))
    }

    pub(crate) fn new(size: u32, encoding: Encoding) -> Self {
        Self {
            metadata: VersionAndEncoding::new(1, encoding),
            size,
        }
    }

    pub(crate) const fn encoding(&self) -> Encoding {
        self.metadata.encoding()
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (header, rest) = postcard::take_from_bytes::<Self>(bytes).map_err(|e| {
            Error::new(
                decode_error_class(),
                format!("failed to deserialize header: {}", e),
            )
        })?;

        if header.metadata.version() != 1 {
            return Err(Error::new(
                base_error_class(),
                format!("invalid header version: {}", header.metadata.version()),
            ));
        }

        Ok((header, rest))
    }

    pub(crate) fn write_to<'a>(&self, buf: &'a mut [u8]) -> (usize, &'a mut [u8]) {
        let serialized_slice_len = {
            postcard::to_slice(self, buf)
                .expect("failed to serialize header")
                .len()
        };

        (serialized_slice_len, &mut buf[serialized_slice_len..])
    }

    pub(crate) fn write_varint_to<'a>(&self, buf: &'a mut [u8]) -> (usize, &'a mut [u8]) {
        let mut val = self.size;
        let mut index = 0;

        loop {
            let byte = (val & 0x7f) as u8;
            val >>= 7;

            if val == 0 {
                buf[index] = byte;
                index += 1;
                break;
            } else {
                buf[index] = byte | 0x80;
                index += 1;
            }
        }

        (index, &mut buf[index..])
    }
}
