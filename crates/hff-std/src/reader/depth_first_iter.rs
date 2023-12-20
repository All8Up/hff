use crate::{Hff, TableView};

/// A depth first iterator over hff content.
pub struct DepthFirstIter<'a> {
    /// The hff structure we are following.
    hff: &'a Hff,
    /// The current index of iteration.
    index: Option<usize>,
    /// Stack to maintain the depth.
    depth: Vec<usize>,
}

impl<'a> DepthFirstIter<'a> {
    /// Create a depth first iterator over tables in the tree.
    pub fn new(hff: &'a Hff) -> DepthFirstIter {
        Self {
            hff,
            index: Some(0),
            depth: vec![],
        }
    }
}

impl<'a> Iterator for DepthFirstIter<'a> {
    // Depth and table view.
    type Item = (usize, TableView<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut index) = self.index.take() {
            let view = TableView::new(self.hff, index);
            let depth = self.depth.len();

            index += 1;
            if view.child_count() > 0 {
                self.depth.push(view.child_count());
                self.index = Some(index);
            } else {
                if index < self.hff.tables_array().len() {
                    loop {
                        let last = self.depth.pop().unwrap();
                        if last > 1 {
                            self.depth.push(last - 1);
                            self.index = Some(index);
                            break;
                        }
                    }
                }
            }

            Some((depth, view))
        } else {
            None
        }
    }
}
