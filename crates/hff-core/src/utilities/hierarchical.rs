use crate::{Ecc, Endian, Error, Result, BE, LE, NE};
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// A simple hierarchical storage system for strings.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Hierarchical {
    /// The primary string key for this level of the hierarchy.
    key: String,
    /// The content of this level.
    content: Vec<String>,
    /// The optional children.
    children: Vec<Hierarchical>,
}

impl Hierarchical {
    /// Ecc identifier type.
    const ID: Ecc = Ecc::new("STR_HIER");

    /// Create a new hierarchical structure.
    pub fn new<T: Into<String>>(key: T, content: Vec<String>, children: Vec<Self>) -> Self {
        Self {
            key: key.into(),
            content,
            children,
        }
    }

    /// Get the key value.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the key mutably.
    pub fn key_mut(&mut self) -> &mut String {
        &mut self.key
    }

    /// Get the content.
    pub fn content(&self) -> &[String] {
        &self.content
    }

    /// Get the content mutably.
    pub fn content_mut(&mut self) -> &mut Vec<String> {
        &mut self.content
    }

    /// Push a new content item.
    pub fn push<T: Into<String>>(&mut self, item: T) {
        self.content.push(item.into());
    }

    /// Get the children.
    pub fn children(&self) -> &[Self] {
        &self.children
    }

    /// Get the children mutably.
    pub fn children_mut(&mut self) -> &mut Vec<Self> {
        &mut self.children
    }

    /// Push a child hierarchy.
    pub fn push_child(&mut self, item: Self) {
        self.children.push(item);
    }

    /// Convert the hierarhcy into a byte buffer.
    pub fn to_bytes<E: ByteOrder>(self) -> Result<Vec<u8>> {
        // Create the output buffer to write to.
        let mut bytes = vec![];
        let writer: &mut dyn Write = &mut bytes;

        // Write the identification ID.
        Self::ID.write::<E>(writer)?;

        // Write the structure.
        self.write_structure::<E>(writer)?;

        // Bytes is now filled and nothing went wrong.
        // Likely the only real issues possible are running out of memory
        // due to incorrect content or strings > 65k.
        Ok(bytes)
    }

    /// Write a hierarchical structure recursively.
    fn write_structure<E: ByteOrder>(self, writer: &mut dyn Write) -> Result<()> {
        // Write the key value.
        Self::write_string::<E>(&self.key, writer)?;

        // Write the count of content strings.
        writer.write_u32::<E>(self.content.len() as u32)?;
        // Write each content string.
        for s in self.content {
            Self::write_string::<E>(&s, writer)?;
        }

        // Write each child.
        // Write the count of children.
        writer.write_u32::<E>(self.children.len() as u32)?;
        for c in self.children {
            c.write_structure::<E>(writer)?;
        }

        Ok(())
    }

    // Write a string into the given stream.
    fn write_string<E: ByteOrder>(value: &str, writer: &mut dyn Write) -> Result<()> {
        // Write the key value.
        if value.len() > core::u16::MAX as usize {
            return Err(Error::Invalid("String length greater than max u16!".into()));
        }
        writer.write_u16::<E>(value.len() as u16)?;
        writer.write_all(value.as_bytes())?;

        Ok(())
    }

    /// Create a hierarchical structure from the given bytes.
    /// TODO: Should probably just give this a reader rather than taking
    /// the bytes directly..  Hmmm
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self> {
        // Create a reader.
        let reader: &mut dyn Read = &mut bytes;

        // Detect endian by reading in local endian.
        let id = Ecc::from(reader.read_u64::<NE>()?);
        match id.endian(Self::ID) {
            Some(Endian::Big) => Ok(Self::from_reader::<BE>(reader)?),
            Some(Endian::Little) => Ok(Self::from_reader::<LE>(reader)?),
            None => Err(Error::Invalid("Not a valid hierarchical.".into())),
        }
    }

    // Read the hierarchy with the given endian.
    fn from_reader<E: ByteOrder>(reader: &mut dyn Read) -> Result<Self> {
        // Read the key.
        let key = Self::read_string::<E>(reader)?;

        // Read the content.
        let content_count = reader.read_u32::<E>()?;
        let mut content = vec![];
        for _ in 0..content_count {
            content.push(Self::read_string::<E>(reader)?);
        }

        // Read the children.
        let child_count = reader.read_u32::<E>()?;
        let mut children = vec![];
        for _ in 0..child_count {
            children.push(Self::from_reader::<E>(reader)?);
        }

        Ok(Self {
            key,
            content,
            children,
        })
    }

    // Read a string from the given reader.
    fn read_string<E: ByteOrder>(reader: &mut dyn Read) -> Result<String> {
        let len = reader.read_u16::<E>()?;
        let mut buffer = vec![0; len as usize];
        reader.read_exact(buffer.as_mut_slice())?;

        Ok(std::str::from_utf8(buffer.as_slice())?.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serialization() {
        let test = Hierarchical::new(
            "Bah",
            vec![String::from("1"), String::from("2")],
            vec![Hierarchical::new(
                "Humbug",
                vec![String::from("3"), String::from("4")],
                vec![],
            )],
        );

        let bytes = test.clone().to_bytes::<NE>().unwrap();
        let result = Hierarchical::from_bytes(bytes.as_slice()).unwrap();
        assert_eq!(test, result);
    }
}
