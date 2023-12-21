use super::{ChunkArray, ChunkDesc, DataArray, TableArray};
use crate::{DataSource, Ecc, Error, Result};
use hff_core::{Chunk, Table};

/// Description of a table.
#[derive(Debug)]
pub struct TableDesc {
    /// The primary identifier.
    primary: Ecc,
    /// The secondary identifier.
    secondary: Ecc,
    /// The metadata for the table.
    metadata: Option<Box<dyn DataSource>>,
    /// The chunks attached to the table.
    chunks: Vec<ChunkDesc>,
    /// The child tables.
    children: Vec<TableDesc>,
}

impl TableDesc {
    /// Create a new table description.
    pub fn new(
        primary: Ecc,
        secondary: Ecc,
        metadata: Option<Box<dyn DataSource>>,
        chunks: Vec<ChunkDesc>,
        children: Vec<TableDesc>,
    ) -> Self {
        Self {
            primary,
            secondary,
            metadata,
            chunks,
            children,
        }
    }

    /// Get the primary identifier.
    pub fn primary(&self) -> Ecc {
        self.primary
    }

    /// Get the secondary identifier.
    pub fn secondary(&self) -> Ecc {
        self.secondary
    }

    pub(super) fn flatten(
        self,
        has_sibling: bool,
        tables: &mut TableArray,
        chunks: &mut ChunkArray,
        data: &mut DataArray,
    ) {
        // Adjust and push this table into the arrays.

        // First, record if the table had metadata and push that to the
        // data array if so.
        let had_metadata = if let Some(metadata) = self.metadata {
            data.push(metadata);
            true
        } else {
            false
        };

        // Record the start of the chunks and how many there are.
        let chunk_start = chunks.len();
        let chunk_count = self.chunks.len();

        // Second, push the chunks for this table into the chunk and data arrays.
        for chunk in self.chunks {
            // Push without offset/length, we don't know them at this time.
            chunks.push(Chunk::new(chunk.primary(), chunk.secondary(), 0, 0));
            data.push(chunk.data_source());
        }

        // Record how many tables there are so we can fix up the sibling
        // after pushing children.
        let table_index = tables.len();

        // Push the table.
        tables.push(
            had_metadata,
            Table::create()
                .primary(self.primary)
                .secondary(self.secondary)
                .chunk_index(if chunk_count > 0 {
                    chunk_start as u32
                } else {
                    0
                })
                .chunk_count(chunk_count as u32)
                .child_count(self.children.len() as u32)
                .end(),
        );

        // Now, flatten each child in turn.
        let child_count = self.children.len();
        for (index, child) in self.children.into_iter().enumerate() {
            child.flatten(index < child_count - 1, tables, chunks, data);
        }

        // Adjust the sibling index by the number of children added if required.
        if has_sibling {
            // Compute the sibling.
            let sibling = (tables.len() - table_index) as u32;

            // Get the table we pushed.
            let mut table = &mut tables[table_index];
            *table.1.sibling_mut() = sibling;
        }
    }
}