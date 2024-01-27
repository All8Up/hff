use crate::{Ecc, Error, Result};
use std::ops::{Deref, DerefMut};
use uuid::Uuid;

/// Identifier type as specified in the hff header.
/// This has no impact on behavior at all, it is only a
/// hint to the end user about how to use/view the ID's.
#[repr(u32)]
pub enum IdType {
    /// A simple u128.
    Id = 0,
    /// Dual eight character codes.
    Ecc2 = 1,
    /// A UUID.
    Uuid = 2,
    /// A 16 character code.
    Scc = 3,
    /// An eight character code and a u64.
    EccU64 = 4,
    /// Two u64's.
    U64s = 5,
}

impl Deref for IdType {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Id => &0,
            Self::Ecc2 => &1,
            Self::Uuid => &2,
            Self::Scc => &3,
            Self::EccU64 => &4,
            Self::U64s => &5,
        }
    }
}

impl From<u32> for IdType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Id,
            1 => Self::Ecc2,
            2 => Self::Uuid,
            3 => Self::Scc,
            4 => Self::EccU64,
            5 => Self::U64s,
            _ => panic!("Invalid identifier type in header."),
        }
    }
}

/// An identifier for the tables and chunks.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Identifier(u128);

impl Identifier {
    /// Create a new instance.
    pub fn new(id: u128) -> Self {
        Self(id)
    }

    // Conversions back to specific identifier types.

    /// Convert to a pair of u64's.
    pub fn as_u64s(self) -> (u64, u64) {
        ((self.0 >> 64) as u64, self.0 as u64)
    }

    /// Convert to dual ecc's.
    pub fn as_ecc2(self) -> (Ecc, Ecc) {
        let (l, r) = self.as_u64s();
        (l.into(), r.into())
    }

    /// Convert to an Uuid.
    pub fn as_uuid(self) -> Uuid {
        Uuid::from_u128(self.0)
    }

    /// Convert to a 16 character code.  (AKA: [u8; 16] right now.)
    pub fn as_scc(self) -> [u8; 16] {
        #[cfg(target_endian = "little")]
        {
            self.0.to_le_bytes()
        }

        #[cfg(target_endian = "big")]
        {
            self.0.to_be_bytes()
        }
    }

    /// Convert to an Ecc and u64.
    pub fn as_eccu64(self) -> (Ecc, u64) {
        let (ecc, value) = self.as_u64s();
        (Ecc::from(ecc), value)
    }
}

impl Deref for Identifier {
    type Target = u128;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Identifier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Into<u128> for Identifier {
    fn into(self) -> u128 {
        self.0
    }
}

impl Into<(Ecc, Ecc)> for Identifier {
    fn into(self) -> (Ecc, Ecc) {
        self.as_ecc2()
    }
}

impl Into<Uuid> for Identifier {
    fn into(self) -> Uuid {
        self.as_uuid()
    }
}

impl Into<[u8; 16]> for Identifier {
    fn into(self) -> [u8; 16] {
        self.as_scc()
    }
}

impl Into<(Ecc, u64)> for Identifier {
    fn into(self) -> (Ecc, u64) {
        self.as_eccu64()
    }
}

impl Into<(u64, u64)> for Identifier {
    fn into(self) -> (u64, u64) {
        self.as_u64s()
    }
}

// Several infallible conversions first.

// For IdType::Id
impl TryFrom<u128> for Identifier {
    type Error = Error;

    fn try_from(value: u128) -> Result<Self> {
        Ok(Self(value))
    }
}

// For IdType::Ecc2
impl TryFrom<(Ecc, Ecc)> for Identifier {
    type Error = Error;

    fn try_from(value: (Ecc, Ecc)) -> Result<Self> {
        let primary: u64 = *value.0;
        let secondary: u64 = *value.1;
        let value = (primary as u128) << 64 | (secondary as u128);
        Ok(Self(value))
    }
}

// For IdType::Uuid
impl TryFrom<Uuid> for Identifier {
    type Error = Error;

    fn try_from(value: Uuid) -> Result<Self> {
        Ok(Self(value.as_u128()))
    }
}

// For IdType::Scc
impl TryFrom<[u8; 16]> for Identifier {
    type Error = Error;

    fn try_from(value: [u8; 16]) -> Result<Self> {
        #[cfg(target_endian = "little")]
        let value = u128::from_le_bytes(value);
        #[cfg(target_endian = "big")]
        let value = u128::from_be_bytes(value);

        Ok(Self(value))
    }
}

// For IdType::EccU64
impl TryFrom<(Ecc, u64)> for Identifier {
    type Error = Error;

    fn try_from(value: (Ecc, u64)) -> Result<Self> {
        let ecc: u64 = *value.0;
        let value = (ecc as u128) << 64 | (value.1 as u128);
        Ok(Self(value))
    }
}

// For IdType::U64s
impl TryFrom<(u64, u64)> for Identifier {
    type Error = Error;

    fn try_from(value: (u64, u64)) -> Result<Self> {
        let value = (value.0 as u128) << 64 | (value.1 as u128);
        Ok(Self(value))
    }
}

// Some helper conversions for ease of use.  These are actually fallible.

impl TryFrom<(&str, &str)> for Identifier {
    type Error = Error;

    fn try_from(value: (&str, &str)) -> Result<Self> {
        let primary: Ecc = value.0.try_into()?;
        let secondary: Ecc = value.1.try_into()?;
        Ok((primary, secondary).try_into()?)
    }
}

impl TryFrom<(&str, u64)> for Identifier {
    type Error = Error;

    fn try_from(value: (&str, u64)) -> Result<Self> {
        let ecc: Ecc = value.0.try_into()?;
        Ok((ecc, value.1).try_into()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecc2() {
        let identifier: Identifier = (123, 456).try_into().unwrap();
        let (_123, _456) = identifier.as_u64s();
        assert_eq!(_123, 123);
        assert_eq!(_456, 456);
    }
}
