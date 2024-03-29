use super::{ChunkDesc, DataSource, TableDesc};
use crate::{Error, Identifier, Result};

/// Builder for tables.
#[derive(Debug)]
pub struct TableBuilder<'a> {
    /// The table identifier.
    identifier: Identifier,
    /// Optional metadata associated with the table.
    metadata: Option<DataSource<'a>>,
    /// Chunks associated with the table..
    chunks: Vec<ChunkDesc<'a>>,
    /// Child tables under this table.
    children: Vec<TableBuilder<'a>>,
}

impl<'a> TableBuilder<'a> {
    /// Create a new table builder instance.
    pub fn new(identifier: Identifier) -> Self {
        Self {
            identifier,
            metadata: None,
            chunks: vec![],
            children: vec![],
        }
    }

    /// Set the metadata for this table.
    pub fn metadata<T>(mut self, content: T) -> Result<Self>
    where
        T: TryInto<DataSource<'a>, Error = Error>,
    {
        self.metadata = Some(content.try_into()?);
        Ok(self)
    }

    /// Set the child tables for this table.
    pub fn children(mut self, children: impl IntoIterator<Item = TableBuilder<'a>>) -> Self {
        self.children = children.into_iter().collect::<Vec<_>>();
        self
    }

    /// Set the chunks associated with this table.
    pub fn chunks(mut self, content: impl IntoIterator<Item = ChunkDesc<'a>>) -> Self {
        self.chunks = content.into_iter().collect::<Vec<_>>();
        self
    }

    /// Finish building the table.
    pub fn finish(self) -> TableDesc<'a> {
        TableDesc::new(
            self.identifier,
            self.metadata,
            self.chunks,
            self.children
                .into_iter()
                .map(|desc| desc.finish())
                .collect(),
        )
    }

    /// Get the identifier.
    pub fn identifier(&self) -> Identifier {
        self.identifier
    }
}
