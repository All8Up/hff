use super::TableDesc;
use hff_core::Ecc;

/// Build a table.
#[derive(Debug, Clone)]
pub struct TableBuilder(TableDesc);

impl TableBuilder {
    /// Create a new instance.
    pub(super) fn new(primary: Ecc, secondary: Ecc) -> Self {
        Self(TableDesc::new(primary, secondary))
    }

    /// Add a child table to the current table.
    pub fn table(&mut self, content: TableDesc) {
        self.0.push_table(content);
    }

    /// Add a chunk entry to the current table.
    pub fn chunk(&mut self) {}

    /// End building the table.
    pub fn end(self) -> TableDesc {
        self.0
    }
}
