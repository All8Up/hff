use super::{Hff, TableView};
use std::fmt::Debug;

/// A depth first iterator over hff content.
pub struct DepthFirstIter<'a, T: Debug> {
    /// The hff structure we are following.
    hff: &'a Hff<T>,
    /// The current index of iteration.
    index: usize,
    /// Stack of expected siblings at each depth.
    count: Vec<usize>,
}

impl<'a, T: Debug> DepthFirstIter<'a, T> {
    /// Create a depth first iterator over tables in the tree.
    pub fn new(hff: &'a Hff<T>) -> DepthFirstIter<'a, T> {
        Self {
            hff,
            index: 0,
            count: vec![],
        }
    }
}

impl<'a, T: Debug> Iterator for DepthFirstIter<'a, T> {
    // Depth and table view.
    type Item = (usize, TableView<'a, T>);

    fn next(&mut self) -> Option<Self::Item> {
        let tables = self.hff.tables_array();

        if self.index < tables.len() {
            // We have more data.
            let table = &tables[self.index];

            // Remove stack entries which have expired.
            while let Some(top) = self.count.pop() {
                if top > 0 {
                    self.count.push(top - 1);
                    break;
                }
            }

            // Store the current depth before adding children.
            let depth = self.count.len();

            // If the table has children, add to the stack.
            if table.child_count() > 0 {
                self.count.push(table.child_count() as usize);
            }

            let view = TableView::new(self.hff, self.index);
            self.index += 1;

            Some((depth, view))
        } else {
            // We're done.
            None
        }
    }
}
