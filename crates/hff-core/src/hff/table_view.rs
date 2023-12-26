use crate::{ChunkIter, Ecc, Hff, TableIter};
use std::fmt::Debug;

/// View of a table.
pub struct TableView<'a> {
    hff: &'a Hff,
    index: usize,
}

impl<'a> PartialEq for TableView<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.hff == other.hff && self.index == other.index
    }
}

impl<'a> Debug for TableView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.hff.tables_array()[self.index])
    }
}

impl<'a> TableView<'a> {
    /// Create a new TableView.
    pub(super) fn new(hff: &'a Hff, index: usize) -> Self {
        Self { hff, index }
    }

    /// Get the hff container we're built from.
    pub fn hff(&self) -> &Hff {
        self.hff
    }

    /// Get the current index into the tables.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Get the primary identifier.
    pub fn primary(&self) -> Ecc {
        self.hff.tables_array()[self.index].primary()
    }

    /// Get the secondary identifier.
    pub fn secondary(&self) -> Ecc {
        self.hff.tables_array()[self.index].secondary()
    }

    /// Get the count of child tables.
    pub fn child_count(&self) -> usize {
        self.hff.tables_array()[self.index].child_count() as usize
    }

    /// Get an iterator to the child tables.
    pub fn iter(&self) -> TableIter<'a> {
        TableIter::new(self.hff, self.index + 1)
    }

    /// Get an iterator of the chunks.
    pub fn chunks(&self) -> ChunkIter<'a> {
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
