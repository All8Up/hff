use hff_core::{
    byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt},
    Ecc, Endian, Error, Result, BE, LE, NE,
};
use std::{
    io::{Read, Write},
    ops::{Deref, DerefMut},
};

/// A simple helper to store a vector of strings as a chunk
/// or metadata within an hff file.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StringVec {
    /// The vector of strings.
    strings: Vec<String>,
}

impl StringVec {
    /// Ecc identifier type.
    const ID: Ecc = Ecc::new("STR_VEC");

    /// Create a new empty string vector.
    pub fn new() -> Self {
        Self { strings: vec![] }
    }

    /// Get the string vector as a byte vector
    /// for storage.
    pub fn to_bytes<E: ByteOrder>(self) -> Result<Vec<u8>> {
        let mut bytes = vec![];
        let writer: &mut dyn Write = &mut bytes;

        Self::ID.write::<E>(writer)?;
        writer.write_u64::<E>(self.strings.len() as u64)?;

        for s in self.strings {
            let b = s.as_bytes();
            writer.write_u64::<E>(b.len() as u64)?;
            writer.write_all(b)?;
        }

        Ok(bytes)
    }

    /// Make a string vector out of the given bytes.
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self> {
        // Detect the endian via the initial ID.
        let reader: &mut dyn Read = &mut bytes;
        // Ecc's are written as u64, so we read it back in
        // native endian and see which endian it was actually
        // in.  (NOTE: Symetric ID's would not work for this
        // so don't try it if you had an id like "ssssssss"
        // since there is no way to detect endianness.)
        let id = Ecc::from(reader.read_u64::<NE>()?);
        match id.endian(Self::ID) {
            Some(endian) => match endian {
                Endian::Big => Self::from_bytes_endian::<BE>(reader),
                Endian::Little => Self::from_bytes_endian::<LE>(reader),
            },
            None => Err(Error::Invalid("Not a string vector.".into())),
        }
    }

    /// Helper for from bytes which deals with endian.
    fn from_bytes_endian<E: ByteOrder>(reader: &mut dyn Read) -> Result<Self> {
        let count = reader.read_u64::<E>()?;

        let mut strings = StringVec::new();
        for _ in 0..count {
            let len = reader.read_u64::<E>()?;
            let mut s = vec![0; len as usize];
            reader.read_exact(&mut s)?;
            strings.push(std::str::from_utf8(&s)?.to_string());
        }

        Ok(strings)
    }
}

impl<I, V> From<I> for StringVec
where
    I: Iterator<Item = V>,
    V: AsRef<str>,
{
    fn from(value: I) -> Self {
        Self {
            strings: value.map(|s| s.as_ref().to_owned()).collect(),
        }
    }
}

impl Deref for StringVec {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.strings
    }
}

impl DerefMut for StringVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.strings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let test_data = ["This", "is", "some", "test", "data."];
        let strings: StringVec = test_data.clone().iter().into();
        let bytes = strings.to_bytes::<LE>().unwrap();

        let result = StringVec::from_bytes(&bytes).unwrap();
        assert!(
            test_data
                .iter()
                .zip(result.iter())
                .map(|(l, r)| if l == r { 0 } else { 1 })
                .sum::<usize>()
                == 0
        );
    }
}
