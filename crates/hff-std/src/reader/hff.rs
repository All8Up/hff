use crate::{DepthFirstIter, TableIter};
use hff_core::{Chunk, Header, Table};
use std::{fmt::Debug, mem::size_of};

/// The Hff structure data.  This is an immutable representation of the
/// content of an Hff stream.
#[derive(Debug, Clone, PartialEq)]
pub struct Hff {
    /// The tables found in the header structure.
    tables: Vec<Table>,
    /// The chunks found within the header structure.
    chunks: Vec<Chunk>,
}

impl Hff {
    /// Create a new Hff wrapper.
    pub fn new(tables: impl Into<Vec<Table>>, chunks: impl Into<Vec<Chunk>>) -> Self {
        Self {
            tables: tables.into(),
            chunks: chunks.into(),
        }
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
    pub(super) fn tables_array(&self) -> &[Table] {
        &self.tables
    }

    /// Get access to the chunk array.
    pub(super) fn chunks_array(&self) -> &[Chunk] {
        &self.chunks
    }
}
