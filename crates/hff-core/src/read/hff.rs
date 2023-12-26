use crate::{read::*, Chunk, Header, Semver, Table};
use std::{fmt::Debug, mem::size_of};

/// The Hff structure data.  This is an immutable representation of the
/// content of an Hff stream.
#[derive(Debug, Clone, PartialEq)]
pub struct Hff {
    /// Was the structure in native endian?
    native: bool,
    /// The version of the file format.
    version: Semver,
    /// The tables found in the header structure.
    tables: Vec<Table>,
    /// The chunks found within the header structure.
    chunks: Vec<Chunk>,
}

impl Hff {
    /// Create a new Hff wrapper.
    pub fn new(
        header: Header,
        tables: impl Into<Vec<Table>>,
        chunks: impl Into<Vec<Chunk>>,
    ) -> Self {
        Self {
            native: header.is_native_endian(),
            version: header.version(),
            tables: tables.into(),
            chunks: chunks.into(),
        }
    }

    /// Return if the structure of the source was in native endian.
    pub fn is_native_endian(&self) -> bool {
        self.native
    }

    /// Return the version of the file structure the file was read from.
    pub fn version(&self) -> Semver {
        self.version
    }

    /// Get the offset from the start of the file to the start of the chunk data.
    pub fn offset_to_data(&self) -> usize {
        size_of::<Header>()
            + (size_of::<Table>() * self.tables.len())
            + (size_of::<Chunk>() * self.chunks.len())
    }

    /// Get an iterator over the tables in depth first order.
    pub fn depth_first(&self) -> DepthFirstIter {
        DepthFirstIter::new(self)
    }

    /// Get an iterator over the child tables.
    pub fn tables(&self) -> TableIter {
        TableIter::new(self, 0)
    }

    /// Get access to the table array.
    pub fn tables_array(&self) -> &[Table] {
        &self.tables
    }

    /// Get access to the chunk array.
    pub fn chunks_array(&self) -> &[Chunk] {
        &self.chunks
    }
}
