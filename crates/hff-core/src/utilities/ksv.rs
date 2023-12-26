use super::StringVec;
use crate::{
    byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt},
    Ecc, Endian, Error, Result, BE, LE, NE,
};
use std::{
    collections::BTreeMap,
    io::{copy, Read, Write},
    ops::{Deref, DerefMut},
};

/// A key + string vector container.  Basically
/// just a key+value system except keys are strings
/// and the values are all string vectors.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ksv {
    /// The key+value mapping.
    value_map: BTreeMap<String, StringVec>,
}

impl Ksv {
    /// Ecc identifier type.
    const ID: Ecc = Ecc::new("STR_VEC");

    /// Create a new empty Ksv.
    pub fn new() -> Self {
        Self {
            value_map: BTreeMap::new(),
        }
    }

    /// Convert the Ksv to bytes.
    pub fn to_bytes<E: ByteOrder>(self) -> Result<Vec<u8>> {
        let mut bytes = vec![];
        let writer: &mut dyn Write = &mut bytes;

        Self::ID.write::<E>(writer)?;
        writer.write_u64::<E>(self.value_map.len() as u64)?;

        for (k, v) in self.value_map {
            // Write the key.
            let kbytes = k.as_bytes();
            writer.write_u64::<E>(kbytes.len() as u64)?;
            writer.write_all(kbytes)?;

            // Write the string vector.
            let vbytes = v.to_bytes::<E>()?;
            writer.write_u64::<E>(vbytes.len() as u64)?;
            copy(&mut vbytes.as_slice(), writer)?;
        }

        Ok(bytes)
    }

    /// Create a Ksv from the given bytes.
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self> {
        // Detect the endian via the initial ID.
        let reader: &mut dyn Read = &mut bytes;
        // Ecc's are written as u64, so we read it back in
        // native endian and see which endian it was actually
        // in.  (NOTE: Symetric ID's would not work for this
        // so don't try it if you had an id like "ssssssss"
        // since there is no way to detect endianess.)
        let id = Ecc::from(reader.read_u64::<NE>()?);
        match id.endian(Self::ID) {
            Some(endian) => match endian {
                Endian::Big => Self::from_bytes_endian::<BE>(reader),
                Endian::Little => Self::from_bytes_endian::<LE>(reader),
            },
            None => Err(Error::Invalid("Not a string vector.".into())),
        }
    }

    /// Helper to read in proper endian.
    fn from_bytes_endian<E: ByteOrder>(reader: &mut dyn Read) -> Result<Self> {
        let count = reader.read_u64::<E>()?;
        let mut result = Self::new();
        for _ in 0..count {
            // Read the key.
            let len = reader.read_u64::<E>()?;
            let mut s = vec![0; len as usize];
            reader.read_exact(&mut s)?;

            // Read the string vector value.
            let len = reader.read_u64::<E>()?;
            let mut v = vec![0; len as usize];
            reader.read_exact(&mut v)?;
            let v = StringVec::from_bytes(&v)?;

            // And put in the result.
            result.insert(std::str::from_utf8(&s)?.to_owned(), v);
        }

        Ok(result)
    }
}

impl<I, S> From<I> for Ksv
where
    I: Iterator<Item = (S, StringVec)>,
    S: AsRef<str>,
{
    fn from(value: I) -> Self {
        let mut result = Self::new();
        for (k, v) in value {
            result.insert(k.as_ref().to_owned(), v);
        }
        result
    }
}

impl Deref for Ksv {
    type Target = BTreeMap<String, StringVec>;

    fn deref(&self) -> &Self::Target {
        &self.value_map
    }
}

impl DerefMut for Ksv {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let test_data: &[(&'static str, StringVec)] = &[
            ("1", ["this", "is"].iter().into()),
            ("2", ["test", "data"].iter().into()),
            ("3", ["for", "Ksv", "containers."].iter().into()),
        ];
        let test: Ksv = test_data.to_owned().into_iter().into();
        let bytes = test.to_bytes::<LE>().unwrap();
        let result = Ksv::from_bytes(&bytes).unwrap();

        assert_eq!(Ksv::from(test_data.to_owned().into_iter()), result);
    }
}
