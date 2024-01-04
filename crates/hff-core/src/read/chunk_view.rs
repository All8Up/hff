use super::Hff;
use crate::{ContentInfo, Ecc};
use std::fmt::Debug;

/// A view to a chunk.
#[derive(Copy, Clone)]
pub struct ChunkView<'a, T: Debug> {
    hff: &'a Hff<T>,
    index: usize,
}

impl<'a, T: Debug> Debug for ChunkView<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.hff.chunks_array()[self.index])
    }
}

impl<'a, T: Debug> ChunkView<'a, T> {
    /// Create a new view.
    pub fn new(hff: &'a Hff<T>, index: usize) -> Self {
        Self { hff, index }
    }

    /// Get the hff this was built from.
    pub fn hff(&self) -> &Hff<T> {
        self.hff
    }

    /// Get the current index.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Get the primary identifier.
    pub fn primary(&self) -> Ecc {
        self.hff.chunks_array()[self.index].primary()
    }

    /// Get the secondary identifier.
    pub fn secondary(&self) -> Ecc {
        self.hff.chunks_array()[self.index].secondary()
    }

    /// Get the size of the data in the chunk.
    pub fn size(&self) -> usize {
        self.hff.chunks_array()[self.index].length() as usize
    }
}

impl<'a, T: Debug> ContentInfo for ChunkView<'a, T> {
    fn len(&self) -> u64 {
        self.hff.chunks_array()[self.index].length()
    }

    fn offset(&self) -> u64 {
        self.hff.chunks_array()[self.index].offset()
    }
}
