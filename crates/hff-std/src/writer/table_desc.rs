use super::{ChunkDesc, DataBuilder};
use crate::DataSource;
use hff_core::{Chunk, Ecc, Error, Header, Result, Table};
use std::{fmt::Debug, mem::size_of};

/// Helper structure to build a flattened table tree.
pub(crate) struct Flattened {
    /// The root tree at this level.
    pub root: Table,
    /// The children of this tree.
    pub children: Vec<Table>,
    /// The metadata and chunk data for the root and children.
    pub data_builder: DataBuilder,
    /// The tracking information for building the chunk array.
    /// (primary, secondary, length, offset)
    pub chunks: Vec<Chunk>,
}

impl Debug for Flattened {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("Flattened");
        s.field("0: ", &self.root);
        for (index, child) in self.children.iter().enumerate() {
            s.field(&(index + 1).to_string(), &child);
        }
        for (index, chunk) in self.chunks.iter().enumerate() {
            s.field(&index.to_string(), chunk);
        }
        s.field("Data", &self.data_builder);
        s.finish()
    }
}

impl Flattened {
    /// Create a new flattened structure with the given
    /// information.
    pub fn with(
        root: Table,
        children: Vec<Table>,
        chunks: Vec<Chunk>,
        data_builder: DataBuilder,
    ) -> Self {
        Self {
            root,
            children,
            data_builder,
            chunks,
        }
    }

    /// Finish the structure.
    pub fn finish(mut self) -> (Vec<Table>, Vec<Chunk>, DataBuilder) {
        // Adjust root to take into account for itself in regards to siblings.
        *self.root.sibling_mut() += 1;

        // Finish flattening.
        let mut tables = vec![self.root];
        tables.extend(self.children);

        // Get the size of the header data before the chunk data.
        let offset = size_of::<Header>()
            + size_of::<Table>() * tables.len()
            + size_of::<Chunk>() * self.chunks.len();

        // Adjust all offsets in tables and chunks to account for the
        // header.
        for table in &mut tables {
            *table.metadata_offset_mut() += offset as u64;
        }
        for chunk in &mut self.chunks {
            *chunk.offset_mut() += offset as u64;
        }

        // Return it all.
        (tables, self.chunks, self.data_builder)
    }
}

/// A table description.
#[derive(Debug)]
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
    metadata: Option<Box<dyn DataSource>>,
}

impl TableDesc {
    /// Create a new table description.
    pub fn new(primary: Ecc, secondary: Ecc) -> Self {
        Self {
            primary,
            secondary,
            children: vec![],
            chunks: vec![],
            metadata: None,
        }
    }

    /// Get the table count.
    pub fn table_count(&self) -> usize {
        let mut count = 0;
        for child in &self.children {
            count += child.table_count();
        }
        // +1 for the table which self itself represents.
        count + 1
    }

    /// Push a child table.
    pub(crate) fn push_table(&mut self, child: TableDesc) {
        self.children.push(child);
    }

    /// Get the chunk count.
    pub fn chunk_count(&self) -> usize {
        let mut count = self.chunks.len();
        for child in &self.children {
            count += child.chunk_count();
        }
        count
    }

    /// Push a chunk onto the table.
    pub(crate) fn push_chunk(&mut self, chunk: ChunkDesc) {
        self.chunks.push(chunk);
    }

    /// Add metadata.
    pub fn metadata(&mut self, data_source: Box<dyn DataSource>) -> Result<()> {
        if self.metadata.is_none() {
            self.metadata = Some(data_source);
            Ok(())
        } else {
            Err(Error::Invalid(
                "Tables can only contain a single metadata entry.".into(),
            ))
        }
    }

    /// Flatten the table hierarchy.
    pub(crate) fn flatten_tables(self) -> Result<Flattened> {
        // Create a new flattened structure to contain this table and
        // its children.
        let mut data_builder = DataBuilder::new();
        let metadata_length = if let Some(mut metadata) = self.metadata {
            let length = if let Some(length) = metadata.len() {
                length
            } else {
                metadata.prepare()?
            };
            data_builder.push(metadata, length);
            length
        } else {
            0
        };

        // Push the chunk data onto the flattened structure and
        // add to the builder.
        let chunk_count = self.chunks.len();
        let mut chunks = vec![];
        for chunk in self.chunks {
            let primary = chunk.primary();
            let secondary = chunk.secondary();
            let mut data = chunk.data();
            let length = if let Some(length) = data.len() {
                length
            } else {
                data.prepare()?
            };

            // Push the chunk entry.
            chunks.push(Chunk::new(
                primary,
                secondary,
                length,
                data_builder.size_bytes(),
            ));
            // Push the data.
            data_builder.push(data, length);
        }

        // Split the children and their related children.
        let mut children = vec![];
        let child_count = self.children.len();
        for child in self.children {
            // Flatten the children completely.
            let mut child_data = child.flatten_tables()?;

            // Update the root sibling.
            *child_data.root.sibling_mut() = (1 + child_data.children.len()) as u32;

            // Offset and push the root.
            child_data
                .root
                .offset(data_builder.size_bytes(), chunks.len() as u32);
            children.push(child_data.root);

            // Offset the children.
            for table in &mut child_data.children {
                table.offset(data_builder.size_bytes(), chunks.len() as u32);
            }
            // Append the children.
            children.append(&mut child_data.children);

            // Update the child chunk offsets.
            for chunk in &mut child_data.chunks {
                *chunk.offset_mut() += data_builder.size_bytes();
            }

            // Append the child chunks to the total chunks.
            chunks.append(&mut child_data.chunks);

            // Append the child data to the current data.
            data_builder.append(child_data.data_builder);
        }

        Ok(Flattened::with(
            Table::create()
                .primary(self.primary)
                .secondary(self.secondary)
                .metadata_length(metadata_length)
                .metadata_offset(0)
                .child_count(child_count as u32)
                .sibling(children.len() as u32)
                .chunk_count(chunk_count as u32)
                .chunk_index(0)
                .end(),
            children,
            chunks,
            data_builder,
        ))
    }
}
