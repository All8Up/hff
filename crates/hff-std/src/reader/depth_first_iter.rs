use crate::{Hff, TableView};

/// A depth first iterator over hff content.
pub struct DepthFirstIter<'a> {
    /// The hff structure we are following.
    hff: &'a Hff,
    /// The current index of iteration.
    index: usize,
    /// Stack of expected siblings at each depth.
    count: Vec<usize>,
}

impl<'a> DepthFirstIter<'a> {
    /// Create a depth first iterator over tables in the tree.
    pub fn new(hff: &'a Hff) -> DepthFirstIter {
        Self {
            hff,
            index: 0,
            count: vec![],
        }
    }
}

impl<'a> Iterator for DepthFirstIter<'a> {
    // Depth and table view.
    type Item = (usize, TableView<'a>);

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

        /*
        println!("{}: {:?}", self.index, self.sibling);
        while let Some(sibling) = self.sibling.pop() {
            // Get the table we intend to return for information purposes.
            let table = self.hff.tables_array()[self.index];

            if sibling == self.index {
                // We're done with the level.
                // If the current item has more siblings, push that in.
                if table.sibling() > 0 {
                    self.sibling.push(self.index + table.sibling() as usize);
                    break;
                }
            } else {
                // Push the sibling back on the stack.
                self.sibling.push(sibling);
                // If the current table has siblings, push that on the stack.
                if table.sibling() > 0 {
                    self.sibling.push(self.index + table.sibling() as usize);
                }
                break;
            }
        }

        if self.index < self.hff.tables_array().len() {
            let view = TableView::new(self.hff, self.index);

            self.index += 1;
            Some((depth, view))
        } else {
            None
        }
         */
    }
}
