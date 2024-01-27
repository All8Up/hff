use super::{ChunkIter, Hff, TableIter};
use crate::{ContentInfo, Identifier};
use std::fmt::Debug;

/// View of a table.
#[derive(Copy, Clone)]
pub struct TableView<'a, T: Debug> {
    hff: &'a Hff<T>,
    index: usize,
}

impl<'a, T: Debug> Debug for TableView<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.hff.tables_array()[self.index])
    }
}

impl<'a, T: Debug> ContentInfo for TableView<'a, T> {
    fn len(&self) -> u64 {
        self.hff.tables_array()[self.index].metadata_length()
    }
    fn offset(&self) -> u64 {
        self.hff.tables_array()[self.index].metadata_offset()
    }
}

impl<'a, T: Debug> TableView<'a, T> {
    /// Create a new TableView.
    pub fn new(hff: &'a Hff<T>, index: usize) -> Self {
        Self { hff, index }
    }

    /// Get the hff container we're built from.
    pub fn hff(&self) -> &Hff<T> {
        self.hff
    }

    /// Determine if the table has metadata.
    pub fn has_metadata(&self) -> bool {
        self.hff.tables_array()[self.index].metadata_length() > 0
    }

    /// Get the current index into the tables.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Get the identifier.
    pub fn identifier(&self) -> Identifier {
        self.hff.tables_array()[self.index].identifier()
    }

    /// Get the count of child tables.
    pub fn child_count(&self) -> usize {
        self.hff.tables_array()[self.index].child_count() as usize
    }

    /// Get an iterator to the child tables.
    pub fn iter(&self) -> TableIter<'a, T> {
        if self.child_count() > 0 {
            TableIter::new(self.hff, self.index + 1)
        } else {
            TableIter::empty(self.hff)
        }
    }

    /// Get an iterator of the chunks.
    pub fn chunks(&self) -> ChunkIter<'a, T> {
        let table = &self.hff.tables_array()[self.index];
        ChunkIter::new(
            self.hff,
            table.chunk_index() as usize,
            table.chunk_count() as usize,
        )
    }

    /// Get the count of chunks in the table.
    pub fn chunk_count(&self) -> usize {
        self.hff.tables_array()[self.index].chunk_count() as usize
    }
}
