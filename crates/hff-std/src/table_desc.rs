use super::ChunkDesc;
use hff_core::{Ecc, Table};

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
    /// The metadata associated with the table.
    metadata: (),
}

impl TableDesc {
    /// Create a new table description.
    pub fn new(primary: Ecc, secondary: Ecc) -> Self {
        Self {
            primary,
            secondary,
            children: vec![],
            chunks: vec![],
            metadata: (),
        }
    }

    /// Get the table count.
    pub fn table_count(&self) -> usize {
        self.children.len()
    }

    /// Push a child table.
    pub fn push_table(&mut self, child: TableDesc) {
        self.children.push(child);
    }

    /// Get the chunk count.
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Push a chunk onto the table.
    pub fn push_chunk(&mut self, chunk: ChunkDesc) {
        self.chunks.push(chunk);
    }

    /// Flatten the table hierarchy.
    pub fn flatten_tables(&self) -> (Table, Vec<Table>) {
        let mut parent = Table::create()
            // Content identification.
            .primary(self.primary)
            .secondary(self.secondary)
            // Our metadata.
            .metadata_length(0)
            .metadata_offset(0)
            // Our count of children.
            .child_count(self.children.len() as u32)
            // We have no immediate siblings.
            .sibling(0)
            // Our count of chunks.
            .chunk_count(self.chunk_count() as u32)
            // Our chunks always start at 0.
            .chunk_index(0)
            .end();

        (parent, vec![])
    }
}
