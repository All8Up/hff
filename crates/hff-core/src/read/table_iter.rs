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

    /// Create an empty iterator.
    pub fn empty(hff: &'a Hff<T>) -> Self {
        Self { hff, index: None }
    }

    /// Create a table iterator for the child of the current table.
    pub fn children(&self) -> Self {
        if let Some(index) = &self.index {
            if self.hff.tables_array()[*index].child_count() > 0 {
                let next = *index + 1;
                assert!(next < self.hff.tables_array().len());
                return Self::new(self.hff, next);
            }
        }

        // Nothing to iterate.
        Self::empty(self.hff)
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
