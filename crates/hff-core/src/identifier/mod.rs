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
        }
    }
}

/// An identifier for the tables and chunks.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Identifier(u128);

impl Identifier {
    /// Create a new instance.
    pub fn new(id: u128) -> Self {
        Self(id)
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

// Some helper conversions for ease of use.  These are actually fallible.

impl<T: AsRef<str>> TryFrom<(T, T)> for Identifier {
    type Error = Error;

    fn try_from(value: (T, T)) -> Result<Self> {
        let primary: Ecc = value.0.as_ref().try_into()?;
        let secondary: Ecc = value.1.as_ref().try_into()?;
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
