use super::ChunkDesc;
use hff_core::Ecc;

/// A table description.
#[derive(Debug, Clone)]
pub struct TableDesc {
    /// The primary identifier of this table.
    primary: Ecc,
    /// The secondary identifier of this table.
    secondary: Ecc,
    /// The tables which are children of this table.
    children: Vec<TableDesc>,
    /// The chunks attached to this table.
    chunks: Vec<ChunkDesc>,
}

impl TableDesc {
    /// Create a new table description.
    pub fn new(primary: Ecc, secondary: Ecc) -> Self {
        Self {
            primary,
            secondary,
            children: vec![],
            chunks: vec![],
        }
    }

    /// Push a child table.
    pub fn push_table(&mut self, child: TableDesc) {
        self.children.push(child);
    }

    /// Push a chunk onto the table.
    pub fn push_chunk(&mut self, chunk: ChunkDesc) {
        self.chunks.push(chunk);
    }
}
