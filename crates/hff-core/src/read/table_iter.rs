use super::{Hff, TableView};
use std::fmt::Debug;

/// An iterator over tables at a given level.
pub struct TableIter<'a, T: Debug> {
    hff: &'a Hff<T>,
    index: Option<usize>,
}

impl<'a, T: Debug> TableIter<'a, T> {
    /// Create a new table iterator.
    pub fn new(hff: &'a Hff<T>, start: usize) -> Self {
        Self {
            hff,
            index: Some(start),
        }
    }
}

impl<'a, T: Debug> Iterator for TableIter<'a, T> {
    type Item = TableView<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.index.take() {
            let result = Some(TableView::new(self.hff, index));

            let sibling = self.hff.tables_array()[index].sibling() as usize;
            if sibling > 0 && index + sibling < self.hff.tables_array().len() {
                self.index = Some(index + sibling)
            }

            result
        } else {
            None
        }
    }
}
