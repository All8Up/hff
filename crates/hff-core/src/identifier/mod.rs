//use crate::{Ecc, Error, Result};

/// Identifier type.
/// The file can specify the type used during creation
/// if desired.  This does not change the behavior nor
/// how things are matched, it is a utility for purposes
/// of the use case being utilized.
#[repr(u32)]
pub enum IdType {
    /// Pure numeric id's.
    Id = 0,
    /// Dual eight character codes.
    Ecc = 1,
    /// A UUID.
    Uuid = 2,
    /// A 16 character code.
    Scc = 3,
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

// impl TryFrom<u128> for Identifier {
//     type Error = Error;

//     fn try_from(value: u128) -> Result<Self> {
//         Ok(Self(value))
//     }
// }

// impl<T> TryFrom<(T, T)> for Identifier
// where
//     T: TryInto<Ecc>,
//     <T as TryInto<Ecc>>::Error: std::fmt::Debug,
//     Error: From<<T as TryInto<Ecc>>::Error>,
// {
//     type Error = Error;

//     fn try_from(value: (T, T)) -> Result<Self> {
//         Ok(Identifier::Ecc(value.0.try_into()?, value.1.try_into()?))
//     }
// }

// impl TryFrom<uuid::Uuid> for Identifier {
//     type Error = Error;

//     fn try_from(value: uuid::Uuid) -> std::prelude::v1::Result<Self, Self::Error> {
//         Ok(Identifier::Uuid(value))
//     }
// }
