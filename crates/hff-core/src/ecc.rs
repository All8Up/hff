use crate::Endian;
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::{
    fmt::Debug,
    io::{Error, Read, Write},
    ops::{Deref, DerefMut},
};

/// 8 character code.
#[repr(C)]
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Ecc(
    /// The code for the identifier.
    u64,
);

impl Debug for Ecc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ecc(\"{}\")", self.to_string())
    }
}

impl Default for Ecc {
    fn default() -> Self {
        Self::INVALID
    }
}

impl Ecc {
    /// Constant representing an invalid Ecc.
    pub const INVALID: Ecc = Self(0);
    /// The file header magic value.
    pub const HFF_MAGIC: Ecc = Self::new("HFF-2023");

    /// Create a new identifier.
    /// Panics if the provided string is empty or if the number of
    /// 'bytes' which represent the string won't fit into a u64.
    /// i.e. the utf8 representation must be less than 9 bytes.
    pub const fn new(name: &str) -> Self {
        let bytes = name.as_bytes();
        let count = bytes.len();

        if count == 0 || count > 8 {
            panic!("Invalid ECC string.");
        }

        // This is rather cheesy but it is desirable for 'new' to
        // be const and this seemed like the way to make that happen.
        union BytesU64 {
            value: u64,
            bytes: [u8; 8],
        }

        let mut converter = BytesU64 { value: 0 };
        unsafe {
            converter.bytes[0] = bytes[0];
            converter.bytes[1] = if count > 1 { bytes[1] } else { 0 };
            converter.bytes[2] = if count > 2 { bytes[2] } else { 0 };
            converter.bytes[3] = if count > 3 { bytes[3] } else { 0 };
            converter.bytes[4] = if count > 4 { bytes[4] } else { 0 };
            converter.bytes[5] = if count > 5 { bytes[5] } else { 0 };
            converter.bytes[6] = if count > 6 { bytes[6] } else { 0 };
            converter.bytes[7] = if count > 7 { bytes[7] } else { 0 };
        }

        Self(unsafe { converter.value })
    }

    /// Check if the Ecc is valid or not.
    pub fn is_valid(&self) -> bool {
        *self != Self::INVALID
    }

    /// Compare the Ecc's in two ways, native and opposing endians.
    /// If equivalent, returns Some with the endianess otherwise None.
    pub fn endian(&self, rhs: Self) -> Option<Endian> {
        if self.0 == rhs.0 {
            Some(crate::NATIVE_ENDIAN)
        } else if self.0 == rhs.0.swap_bytes() {
            Some(crate::OPPOSING_ENDIAN)
        } else {
            None
        }
    }

    /// Swap the endianess of the code.
    pub const fn swap_bytes(&self) -> Self {
        Self(self.0.swap_bytes())
    }

    /// Get the Ecc as a slice.
    pub const fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts((&self.0 as *const u64) as *const u8, 8) }
    }

    /// Get the Ecc as a mutable slice.
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut((&mut self.0 as *mut u64) as *mut u8, 8) }
    }

    /// Read from a stream.
    pub fn read<E: ByteOrder>(reader: &mut dyn Read) -> Result<Self, Error> {
        Ok(Self(reader.read_u64::<E>()?))
    }

    /// Write to a stream.
    pub fn write<E: ByteOrder>(self, writer: &mut dyn Write) -> Result<(), Error> {
        writer.write_u64::<E>(self.0)?;
        Ok(())
    }
}

impl Deref for Ecc {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Ecc {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TryFrom<String> for Ecc {
    type Error = crate::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let bytes = value.as_bytes();
        let count = bytes.len();

        if count > 0 && count < 9 {
            Ok(Ecc::new(value.as_str()))
        } else {
            Err(crate::Error::InvalidEcc(value))
        }
    }
}

impl From<&str> for Ecc {
    fn from(value: &str) -> Self {
        Ecc::new(value)
    }
}

impl From<u64> for Ecc {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<[u8; 8]> for Ecc {
    fn from(value: [u8; 8]) -> Self {
        unsafe { Self(*(value.as_ptr() as *const u64)) }
    }
}

impl ToString for Ecc {
    fn to_string(&self) -> String {
        let mut name = String::with_capacity(8);
        let code = self.as_slice();
        if self.is_valid() {
            for i in 0..8 {
                if code[i].is_ascii() {
                    if code[i] == 0 {
                        break;
                    } else {
                        name.push(code[i] as char);
                    }
                } else {
                    name = format!("{}", self.0);
                    break;
                }
            }
        } else {
            name = "INVALID".into();
        }
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout() {
        // Must be 8 bytes.
        assert_eq!(std::mem::size_of::<Ecc>(), 8);
    }

    #[test]
    fn test_construction() {
        for i in 1..=8 {
            // Create and check the returned string.
            let id = (0..i).into_iter().map(|_| 'x').collect::<String>();
            let code = Ecc::new(&id);
            assert_eq!(code.to_string(), id);

            // Make sure all the remaining bytes are 0's.
            let bytes = code.as_slice();
            for j in i..8 {
                assert_eq!(bytes[j], 0);
            }
        }
    }

    #[test]
    fn test_serialization_le() {
        let mut buffer = vec![];
        let ecc = Ecc::new("test");
        assert!(ecc.write::<crate::LE>(&mut buffer).is_ok());

        let result = Ecc::read::<crate::LE>(&mut buffer.as_slice()).unwrap();
        assert_eq!(ecc, result);
        assert_eq!(result.to_string(), "test");
    }

    #[test]
    fn test_serialization_be() {
        let mut buffer = vec![];
        let ecc = Ecc::new("test");
        assert!(ecc.write::<crate::BE>(&mut buffer).is_ok());

        let result = Ecc::read::<crate::BE>(&mut buffer.as_slice()).unwrap();
        assert_eq!(ecc, result);
        assert_eq!(result.to_string(), "test");
    }

    #[test]
    fn test_endianness() {
        assert_eq!(
            Ecc::HFF_MAGIC.endian(Ecc::HFF_MAGIC),
            Some(crate::NATIVE_ENDIAN)
        );
        assert_eq!(
            Ecc::from(Ecc::HFF_MAGIC.0.swap_bytes()).endian(Ecc::HFF_MAGIC),
            Some(crate::OPPOSING_ENDIAN)
        );
    }
}
