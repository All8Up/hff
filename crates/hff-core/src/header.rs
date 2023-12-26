//! The file header.
use super::Semver;
use crate::{Ecc, Endian, Error, Result, BE, LE, NATIVE_ENDIAN, NE};
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::io::Write;

/// The current version of the format.
pub const FORMAT_VERSION: Semver = Semver::new(0, 1, 0);

/// The file header.
#[repr(C)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Header {
    /// Magic identifier.  Ecc::HFF_MAGIC
    magic: Ecc,
    /// Version of the file format.
    version: Semver,
    /// The overall content type of this file.
    content: Ecc,
    /// Total count of tables in the header.
    table_count: u32,
    /// Total count of chunks in the header.
    chunk_count: u32,
}

impl Header {
    /// Size of the header.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Create a new instance.
    pub fn new(content: Ecc, table_count: u32, chunk_count: u32) -> Self {
        Self {
            magic: Ecc::HFF_MAGIC,
            version: FORMAT_VERSION,
            content,
            table_count,
            chunk_count,
        }
    }

    /// Create a new instance with the given data.
    pub fn with(
        magic: Ecc,
        version: Semver,
        content: Ecc,
        table_count: u32,
        chunk_count: u32,
    ) -> Self {
        Self {
            magic,
            version,
            content,
            table_count,
            chunk_count,
        }
    }

    /// Check that this is a valid file header.
    pub fn is_valid(&self) -> bool {
        match self.magic.endian(Ecc::HFF_MAGIC) {
            Some(endian) => {
                if endian == NATIVE_ENDIAN {
                    self.version == FORMAT_VERSION
                } else {
                    self.version.swap_bytes() == FORMAT_VERSION
                }
            }
            None => false,
        }
    }

    /// Get the magic value.
    pub fn magic(&self) -> Ecc {
        self.magic
    }

    /// Get the container version.
    pub fn version(&self) -> Semver {
        self.version
    }

    /// What's the endian?
    pub fn is_native_endian(&self) -> bool {
        self.magic == Ecc::HFF_MAGIC
    }

    /// Get the table count.
    pub fn table_count(&self) -> u32 {
        self.table_count
    }

    /// Get the chunk count.
    pub fn chunk_count(&self) -> u32 {
        self.chunk_count
    }

    /// Convert the header to a byte vector.
    pub fn to_bytes<E: ByteOrder>(self) -> Result<Vec<u8>> {
        let mut buffer = vec![];
        let writer: &mut dyn Write = &mut buffer;
        self.magic.write::<E>(writer)?;
        self.version.write::<E>(writer)?;
        self.content.write::<E>(writer)?;
        writer.write_u32::<E>(self.table_count)?;
        writer.write_u32::<E>(self.chunk_count)?;
        Ok(buffer)
    }

    /// A test helper.  Swapping the bytes like this only makes sense for
    /// testing because the read adjusts to endianess after reading only
    /// the magic and not the rest.
    #[cfg(test)]
    pub fn swap_bytes(&self) -> Self {
        Self {
            magic: self.magic.swap_bytes(),
            version: self.version.swap_bytes(),
            content: self.content.swap_bytes(),
            table_count: self.table_count.swap_bytes(),
            chunk_count: self.chunk_count.swap_bytes(),
        }
    }
}

impl TryFrom<&[u8]> for Header {
    type Error = crate::Error;

    fn try_from(mut value: &[u8]) -> std::prelude::v1::Result<Self, Self::Error> {
        let reader: &mut dyn std::io::Read = &mut value;

        // Read the magic in native endian.
        let magic = Ecc::read::<NE>(reader)?;

        // Check the endianness and read the remaining data appropriately.
        // NOTE: The magic is stored as whatever form was found so we can
        // detect the original form at a later time.
        match Ecc::HFF_MAGIC.endian(magic.clone()) {
            Some(endian) => match endian {
                Endian::Little => Ok(Header::with(
                    magic,
                    Semver::read::<LE>(reader)?,
                    Ecc::read::<LE>(reader)?,
                    reader.read_u32::<LE>()?,
                    reader.read_u32::<LE>()?,
                )),
                Endian::Big => Ok(Header::with(
                    magic,
                    Semver::read::<BE>(reader)?,
                    Ecc::read::<BE>(reader)?,
                    reader.read_u32::<BE>()?,
                    reader.read_u32::<BE>()?,
                )),
            },
            None => Err(Error::Invalid("Not an HFF file.".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_layout() {
        assert_eq!(std::mem::size_of::<Header>(), 32);
    }

    #[test]
    fn validation() {
        assert!(Header::new(Ecc::new("test"), 0, 0).is_valid());
        assert!(Header::new(Ecc::new("test"), 0, 0).is_native_endian());
        assert!(Header::new(Ecc::new("test"), 0, 0).swap_bytes().is_valid());
        assert!(!Header::new(Ecc::new("test"), 0, 0)
            .swap_bytes()
            .is_native_endian());
    }

    #[test]
    fn serialization() {
        {
            // Create a header, convert to bytes and then recreate from the bytes.
            let header = Header::new("Test".into(), 1, 2);
            let buffer = header.clone().to_bytes::<LE>().unwrap();
            let dup: Header = buffer.as_slice().try_into().unwrap();

            assert_eq!(dup.magic, Ecc::HFF_MAGIC);
            assert_eq!(dup.version, Semver::new(0, 1, 0));
            assert_eq!(dup.content, Ecc::new("Test"));
            assert_eq!(dup.table_count, 1);
            assert_eq!(dup.chunk_count, 2);
        }

        {
            // Create a header, convert to bytes and then recreate from the bytes.
            let header = Header::new("Test".into(), 1, 2);
            let buffer = header.clone().to_bytes::<BE>().unwrap();
            let dup: Header = buffer.as_slice().try_into().unwrap();

            assert_eq!(dup.magic, Ecc::HFF_MAGIC.swap_bytes());
            assert_eq!(dup.version, Semver::new(0, 1, 0));
            assert_eq!(dup.content, Ecc::new("Test"));
            assert_eq!(dup.table_count, 1);
            assert_eq!(dup.chunk_count, 2);
        }
    }
}
