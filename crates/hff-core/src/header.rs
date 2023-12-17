//! The file header.
use super::Semver;
use crate::{Ecc, Endian, Error, Result, NATIVE_ENDIAN};
use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

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
    pub fn version(&self) -> &Semver {
        &self.version
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

    /// Read from a given stream.
    pub fn read(reader: &mut dyn Read) -> Result<Self> {
        let mut magic = [0_u8; 8];
        reader.read_exact(&mut magic)?;

        // Detect the file content endianess.  NOTE: This only describes
        // the file structure itself, the chunk content is "not" considered
        // part of this.  It is up to the user to deal with endianess of
        // the chunks.
        match Ecc::HFF_MAGIC.endian(magic.into()) {
            Some(endian) => match endian {
                Endian::Little => Ok(Self {
                    magic: magic.into(),
                    version: Semver::read::<LittleEndian>(reader)?,
                    content: Ecc::read::<LittleEndian>(reader)?,
                    table_count: reader.read_u32::<LittleEndian>()?,
                    chunk_count: reader.read_u32::<LittleEndian>()?,
                }),
                Endian::Big => Ok(Self {
                    magic: magic.into(),
                    version: Semver::read::<BigEndian>(reader)?,
                    content: Ecc::read::<BigEndian>(reader)?,
                    table_count: reader.read_u32::<BigEndian>()?,
                    chunk_count: reader.read_u32::<BigEndian>()?,
                }),
            },
            None => Err(Error::Invalid("Not an HFF file.".into())),
        }
    }

    /// Write to the given stream.
    pub fn write<E: ByteOrder>(self, writer: &mut dyn Write) -> Result<()> {
        self.magic.write::<E>(writer)?;
        self.version.write::<E>(writer)?;
        self.content.write::<E>(writer)?;
        writer.write_u32::<E>(self.table_count)?;
        writer.write_u32::<E>(self.chunk_count)?;
        Ok(())
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
}
