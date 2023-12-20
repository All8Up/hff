use crate::{ChunkIter, Hff, ReadSeek, TableIter};
use hff_core::{Ecc, Result};
use std::{fmt::Debug, io::SeekFrom};

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

    /// Read the metadata from the given source.
    pub fn metadata(&self, source: &mut dyn ReadSeek) -> Result<Vec<u8>> {
        let table = &self.hff.tables_array()[self.index];
        if table.metadata_length() > 0 {
            source.seek(SeekFrom::Start(table.metadata_offset()))?;
            let mut buffer = vec![0; table.metadata_length() as usize];
            source.read_exact(buffer.as_mut_slice())?;
            Ok(buffer)
        } else {
            Ok(vec![])
        }
    }
}
