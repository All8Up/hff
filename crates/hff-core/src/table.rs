use crate::{Ecc, Result};
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// A table entry in the file format.
/// Tables are 48 bytes in length when stored.
#[repr(C, align(16))]
#[derive(Debug, Copy, Eq, PartialEq, Clone, Hash)]
pub struct Table {
    /// The primary content type of the table.
    primary: Ecc,
    /// The secondary content type of the table.
    secondary: Ecc,
    /// Length of the metadata optionally attached to this table.
    metadata_length: u64,
    /// Absolute offset to the metadata content.
    metadata_offset: u64,
    /// The number of child tables that are owned by this table.
    /// This is direct children only, not the children of children.
    child_count: u32,
    /// The index of the next sibling table.
    sibling: u32,
    /// The index into the chunk table where the first chunk exists.
    chunk_index: u32,
    /// The number of chunks associated with this table or zero.
    chunk_count: u32,
}

impl Default for Table {
    fn default() -> Self {
        Self {
            primary: Ecc::INVALID,
            secondary: Ecc::INVALID,
            metadata_length: 0,
            metadata_offset: 0,
            child_count: 0,
            sibling: 0,
            chunk_index: 0,
            chunk_count: 0,
        }
    }
}

impl Table {
    /// Create a table using the builder.
    pub fn create() -> TableBuilder {
        TableBuilder::new()
    }

    /// Get the primary content type.
    pub fn primary(&self) -> Ecc {
        self.primary
    }

    /// Get the secondary content type.
    pub fn secondary(&self) -> Ecc {
        self.secondary
    }

    /// Get the metadata length.
    pub fn metadata_length(&self) -> u64 {
        self.metadata_length
    }

    /// Get the metadata offset in the file.
    /// This is an absolute offset within the file, i.e. zero based.
    pub fn metadata_offset(&self) -> u64 {
        self.metadata_offset
    }

    /// Get the child table count.
    pub fn child_count(&self) -> u32 {
        self.child_count
    }

    /// Get the index of the next sibling table.
    pub fn sibling(&self) -> u32 {
        self.sibling
    }

    /// Get the index of the first chunk owned by this table.
    pub fn chunk_index(&self) -> u32 {
        self.chunk_index
    }

    /// Get the number of chunks owned by this table.
    pub fn chunk_count(&self) -> u32 {
        self.chunk_count
    }

    /// Read a table from the given stream.
    pub fn read<E: ByteOrder>(reader: &mut dyn Read) -> Result<Self> {
        Ok(Self {
            primary: Ecc::read::<E>(reader)?,
            secondary: Ecc::read::<E>(reader)?,
            metadata_length: reader.read_u64::<E>()?,
            metadata_offset: reader.read_u64::<E>()?,
            child_count: reader.read_u32::<E>()?,
            sibling: reader.read_u32::<E>()?,
            chunk_index: reader.read_u32::<E>()?,
            chunk_count: reader.read_u32::<E>()?,
        })
    }

    /// Write a table to the given stream.
    pub fn write<E: ByteOrder>(self, writer: &mut dyn Write) -> Result<()> {
        self.primary.write::<E>(writer)?;
        self.secondary.write::<E>(writer)?;
        writer.write_u64::<E>(self.metadata_length)?;
        writer.write_u64::<E>(self.metadata_offset)?;
        writer.write_u32::<E>(self.child_count)?;
        writer.write_u32::<E>(self.sibling)?;
        writer.write_u32::<E>(self.chunk_index)?;
        writer.write_u32::<E>(self.chunk_count)?;

        Ok(())
    }
}

/// Build a table.
pub struct TableBuilder {
    table: Table,
}

impl TableBuilder {
    /// Create a new builder.
    fn new() -> Self {
        Self {
            table: Table::default(),
        }
    }

    /// Set the primary content type.
    pub fn primary(mut self, value: impl Into<Ecc>) -> Self {
        self.table.primary = value.into();
        self
    }

    /// Set the secondary content type.
    pub fn secondary(mut self, value: impl Into<Ecc>) -> Self {
        self.table.secondary = value.into();
        self
    }

    /// Set the metadata length.
    pub fn metadata_length(mut self, value: u64) -> Self {
        self.table.metadata_length = value;
        self
    }

    /// Set the metadata offset in the file.
    pub fn metadata_offset(mut self, value: u64) -> Self {
        self.table.metadata_offset = value;
        self
    }

    /// Get the child table count.
    pub fn child_count(mut self, value: u32) -> Self {
        self.table.child_count = value;
        self
    }

    /// Get the index of the next sibling table.
    pub fn sibling(mut self, value: u32) -> Self {
        self.table.sibling = value;
        self
    }

    /// Get the index of the first chunk owned by this table.
    pub fn chunk_index(mut self, value: u32) -> Self {
        self.table.chunk_index = value;
        self
    }

    /// Get the number of chunks owned by this table.
    pub fn chunk_count(mut self, value: u32) -> Self {
        self.table.chunk_count = value;
        self
    }

    /// Finalize the table.
    pub fn end(self) -> Table {
        self.table
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout() {
        assert_eq!(std::mem::size_of::<Table>(), 48);
    }

    #[test]
    fn test_basics() {
        let table = Table::create()
            .primary("test1")
            .secondary(Ecc::INVALID)
            .metadata_length(1)
            .metadata_offset(2)
            .child_count(3)
            .sibling(4)
            .chunk_count(5)
            .chunk_index(6)
            .end();

        assert_eq!(table.primary(), Ecc::new("test1"));
        assert_eq!(table.secondary(), Ecc::INVALID);
        assert_eq!(table.metadata_length(), 1);
        assert_eq!(table.metadata_offset(), 2);
        assert_eq!(table.child_count(), 3);
        assert_eq!(table.sibling(), 4);
        assert_eq!(table.chunk_count(), 5);
        assert_eq!(table.chunk_index(), 6);
    }

    #[test]
    fn test_serialization() {
        let mut buffer = vec![];
        let table = Table::create()
            .primary("test1")
            .secondary(Ecc::INVALID)
            .metadata_length(1)
            .metadata_offset(2)
            .child_count(3)
            .sibling(4)
            .chunk_count(5)
            .chunk_index(6)
            .end();
        assert!(table.write::<crate::LE>(&mut buffer).is_ok());

        let result = Table::read::<crate::LE>(&mut buffer.as_slice()).unwrap();
        assert_eq!(table, result);
    }
}
