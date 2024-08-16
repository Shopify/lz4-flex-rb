use magnus::Error;

use crate::{base_error_class, Encoding};

#[repr(C)]
pub(crate) struct Header {
    pub(crate) version: u8,
    pub(crate) encoding: Encoding, // u8
    pub(crate) __reserved: [u8; 2],
    pub(crate) size: u32,
}

impl Header {
    pub(crate) fn new(size: u32, encoding: Encoding) -> Self {
        Self {
            version: 1,
            encoding,
            __reserved: [0; 2],
            size,
        }
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let version = bytes[0];
        let encoding = Encoding::from_u8(bytes[1])?;
        let __reserved = [bytes[2], bytes[3]];
        let size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

        if version != 1 {
            return Err(Error::new(
                base_error_class(),
                format!("invalid header version: {}", version),
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

    pub(crate) fn write_to<'a>(&self, buf: &'a mut [u8]) -> &'a mut [u8] {
        let header_section = &mut buf[0..8];
        header_section[0] = self.version;
        header_section[1] = self.encoding as u8;
        header_section[2] = self.__reserved[0];
        header_section[3] = self.__reserved[1];
        header_section[4..8].copy_from_slice(&self.size.to_le_bytes());
        &mut buf[8..]
    }
}
