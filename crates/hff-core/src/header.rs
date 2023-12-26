//! The file header.
use super::Semver;
use crate::{Ecc, Result, NATIVE_ENDIAN};
use byteorder::{ByteOrder, WriteBytesExt};
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NE, OP};

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
            let header = Header::new("Test".into(), 1, 2);
            let mut buffer = vec![];
            assert!(header.write::<NE>(&mut buffer).is_ok());
            let dup = Header::read(&mut buffer.as_slice()).unwrap();
            assert_eq!(dup.magic, Ecc::HFF_MAGIC);
            assert_eq!(dup.version, Semver::new(0, 1, 0));
            assert_eq!(dup.content, Ecc::new("Test"));
            assert_eq!(dup.table_count, 1);
            assert_eq!(dup.chunk_count, 2);
        }

        {
            let header = Header::new("Test".into(), 1, 2);
            let mut buffer = vec![];
            assert!(header.write::<OP>(&mut buffer).is_ok());
            let dup = Header::read(&mut buffer.as_slice()).unwrap();
            assert_eq!(dup.magic, Ecc::HFF_MAGIC.swap_bytes());
            assert_eq!(dup.version, Semver::new(0, 1, 0));
            assert_eq!(dup.content, Ecc::new("Test"));
            assert_eq!(dup.table_count, 1);
            assert_eq!(dup.chunk_count, 2);
        }
    }
}
