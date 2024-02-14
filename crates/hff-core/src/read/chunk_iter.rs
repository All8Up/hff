use super::{ChunkView, Hff};
use std::fmt::Debug;

/// Iterator over a table's chunks.
pub struct ChunkIter<'a, T: Debug> {
    hff: &'a Hff<T>,
    current: isize,
    count: usize,
}

impl<'a, T: Debug> ChunkIter<'a, T> {
    /// Create a new chunk iterator.
    pub fn new(hff: &'a Hff<T>, index: usize, count: usize) -> Self {
        Self {
            hff,
            current: index as isize - 1,
            count,
        }
    }

    /// Get the chunk index in the overall table.
    pub fn index(&self) -> usize {
        self.current as usize
    }
}

impl<'a, T: Debug> Iterator for ChunkIter<'a, T> {
    type Item = ChunkView<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            self.count -= 1;
            self.current += 1;
            Some(ChunkView::new(self.hff, self.current as usize))
        } else {
            None
        }
    }
}
