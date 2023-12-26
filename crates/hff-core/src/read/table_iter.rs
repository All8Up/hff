use super::{Hff, TableView};

/// An iterator over tables at a given level.
pub struct TableIter<'a> {
    hff: &'a Hff,
    index: Option<usize>,
}

impl<'a> TableIter<'a> {
    /// Create a new table iterator.
    pub fn new(hff: &'a Hff, start: usize) -> Self {
        Self {
            hff,
            index: Some(start),
        }
    }
}

impl<'a> Iterator for TableIter<'a> {
    type Item = TableView<'a>;

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
