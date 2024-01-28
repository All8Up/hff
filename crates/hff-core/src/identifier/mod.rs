use crate::{Ecc, Error, Result};
use std::ops::{Deref, DerefMut};
use uuid::Uuid;

/// Identifier type as specified in the hff header.
/// This has no impact on behavior at all, it is only a
/// hint to the end user about how to use/view the ID's.
#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum IdType {
    /// A simple u128.
    Id = 0,
    /// Dual eight character codes.
    Ecc2 = 1,
    /// A UUID.
    Uuid = 2,
    /// An array of u8.
    Au8 = 3,
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
            Self::Au8 => &3,
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
            3 => Self::Au8,
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
    /// An invalid identifier.
    pub const INVALID: Self = Self(0);

    /// Create a new instance.
    pub fn new(id: u128) -> Self {
        Self(id)
    }

    /// Create a string from the identifier formatted to the given type.
    pub fn to_string(&self, id_type: IdType) -> String {
        match id_type {
            IdType::Id => format!("{:X}", self.0),
            IdType::Ecc2 => {
                let (p, s): (Ecc, Ecc) = (*self).into();
                format!("{}:{}", p.to_string(), s.to_string())
            }
            IdType::Uuid => {
                let id: Uuid = (*self).into();
                format!("{}", id.to_string())
            }
            IdType::Au8 => {
                let chars: [u8; 16] = (*self).into();
                format!("{:?}", chars)
            }
            IdType::EccU64 => {
                let (ecc, value): (Ecc, u64) = (*self).into();
                format!("{}:{:X}", ecc.to_string(), value)
            }
            IdType::U64s => {
                let (l, r): (u64, u64) = (*self).into();
                format!("{:X}:{:X}", l, r)
            }
        }
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

    /// Convert to an array of u8.
    pub fn as_au8(self) -> [u8; 16] {
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
        self.as_au8()
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
impl From<u128> for Identifier {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

// For IdType::Ecc2
impl From<(Ecc, Ecc)> for Identifier {
    fn from(value: (Ecc, Ecc)) -> Self {
        let primary: u64 = *value.0;
        let secondary: u64 = *value.1;
        let value = (primary as u128) << 64 | (secondary as u128);
        Self(value)
    }
}

// For IdType::Uuid
impl From<Uuid> for Identifier {
    fn from(value: Uuid) -> Self {
        Self(value.as_u128())
    }
}

// For IdType::Scc
impl From<[u8; 16]> for Identifier {
    fn from(value: [u8; 16]) -> Self {
        #[cfg(target_endian = "little")]
        let value = u128::from_le_bytes(value);
        #[cfg(target_endian = "big")]
        let value = u128::from_be_bytes(value);

        Self(value)
    }
}

// For IdType::EccU64
impl From<(Ecc, u64)> for Identifier {
    fn from(value: (Ecc, u64)) -> Self {
        let ecc: u64 = *value.0;
        let value = (ecc as u128) << 64 | (value.1 as u128);
        Self(value)
    }
}

// For IdType::U64s
impl From<(u64, u64)> for Identifier {
    fn from(value: (u64, u64)) -> Self {
        let value = (value.0 as u128) << 64 | (value.1 as u128);
        Self(value)
    }
}

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
