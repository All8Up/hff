use crate::read::{ChunkView, Hff};

/// Iterator over a table's chunks.
pub struct ChunkIter<'a> {
    hff: &'a Hff,
    current: isize,
    count: usize,
}

impl<'a> ChunkIter<'a> {
    /// Create a new chunk iterator.
    pub fn new(hff: &'a Hff, index: usize, count: usize) -> Self {
        Self {
            hff,
            current: index as isize - 1,
            count,
        }
    }
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = ChunkView<'a>;

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
