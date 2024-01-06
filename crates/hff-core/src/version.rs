use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::io::{Error, Read, Write};

/// Version of the file format.
#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Version {
    /// Major version.
    pub major: u16,
    /// Minor version.
    pub minor: u16,
}

impl Version {
    /// Constant representing the byte size of the structure.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Create a semantic versioning instance.
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Get the major version.
    pub const fn major(&self) -> u16 {
        self.major
    }

    /// Get the minor version.
    pub const fn minor(&self) -> u16 {
        self.minor
    }

    /// Return the Version in the opposite endian.
    pub const fn swap_bytes(&self) -> Self {
        Self {
            major: self.major.swap_bytes(),
            minor: self.minor.swap_bytes(),
        }
    }

    /// Read a segment metadata entry from a stream.
    pub fn read<E: ByteOrder>(reader: &mut dyn Read) -> Result<Self, Error> {
        Ok(Self {
            major: reader.read_u16::<E>()?,
            minor: reader.read_u16::<E>()?,
        })
    }

    /// Write segment metadata to a stream.
    pub fn write<E: ByteOrder>(self, writer: &mut dyn Write) -> Result<(), Error> {
        writer.write_u16::<E>(self.major)?;
        writer.write_u16::<E>(self.minor)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout() {
        assert_eq!(std::mem::size_of::<Version>(), 4);
    }

    #[test]
    fn test_serialization_le() {
        let mut buffer = vec![];
        let version = Version::new(1, 2);
        assert!(version.write::<crate::LE>(&mut buffer).is_ok());

        let result = Version::read::<crate::LE>(&mut buffer.as_slice()).unwrap();
        assert_eq!(version, result);
        assert_eq!(result.major(), 1);
        assert_eq!(result.minor(), 2);
    }

    #[test]
    fn test_serialization_be() {
        let mut buffer = vec![];
        let version = Version::new(1, 2);
        assert!(version.write::<crate::BE>(&mut buffer).is_ok());

        let result = Version::read::<crate::BE>(&mut buffer.as_slice()).unwrap();
        assert_eq!(version, result);
        assert_eq!(result.major(), 1);
        assert_eq!(result.minor(), 2);
    }
}
