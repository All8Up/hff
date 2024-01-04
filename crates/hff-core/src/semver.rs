use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::io::{Error, Read, Write};

/// Semantic version.  The 'patch' is u32 only because we want
/// this structure to fit into 64 bits.
#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Semver {
    /// Major version.
    pub major: u16,
    /// Minor version.
    pub minor: u16,
    /// Patch version.
    pub patch: u32,
}

impl Semver {
    /// Constant representing the byte size of the structure.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Create a semantic versioning instance.
    pub const fn new(major: u16, minor: u16, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Get the major version.
    pub const fn major(&self) -> u16 {
        self.major
    }

    /// Get the minor version.
    pub const fn minor(&self) -> u16 {
        self.minor
    }

    /// Get the patch version.
    pub const fn patch(&self) -> u32 {
        self.patch
    }

    /// Return the semver in the opposite endian.
    pub const fn swap_bytes(&self) -> Self {
        Self {
            major: self.major.swap_bytes(),
            minor: self.minor.swap_bytes(),
            patch: self.patch.swap_bytes(),
        }
    }

    /// Read a segment metadata entry from a stream.
    pub fn read<E: ByteOrder>(reader: &mut dyn Read) -> Result<Self, Error> {
        Ok(Self {
            major: reader.read_u16::<E>()?,
            minor: reader.read_u16::<E>()?,
            patch: reader.read_u32::<E>()?,
        })
    }

    /// Write segment metadata to a stream.
    pub fn write<E: ByteOrder>(self, writer: &mut dyn Write) -> Result<(), Error> {
        writer.write_u16::<E>(self.major)?;
        writer.write_u16::<E>(self.minor)?;
        writer.write_u32::<E>(self.patch)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout() {
        assert_eq!(std::mem::size_of::<Semver>(), 8);
    }

    #[test]
    fn test_serialization_le() {
        let mut buffer = vec![];
        let semver = Semver::new(1, 2, 3);
        assert!(semver.write::<crate::LE>(&mut buffer).is_ok());

        let result = Semver::read::<crate::LE>(&mut buffer.as_slice()).unwrap();
        assert_eq!(semver, result);
        assert_eq!(result.major(), 1);
        assert_eq!(result.minor(), 2);
        assert_eq!(result.patch(), 3);
    }

    #[test]
    fn test_serialization_be() {
        let mut buffer = vec![];
        let semver = Semver::new(1, 2, 3);
        assert!(semver.write::<crate::BE>(&mut buffer).is_ok());

        let result = Semver::read::<crate::BE>(&mut buffer.as_slice()).unwrap();
        assert_eq!(semver, result);
        assert_eq!(result.major(), 1);
        assert_eq!(result.minor(), 2);
        assert_eq!(result.patch(), 3);
    }
}
