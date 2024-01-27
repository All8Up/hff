use super::DataSource;
use crate::Identifier;

/// An intermediate chunk description.
#[derive(Debug)]
pub struct ChunkDesc<'a> {
    /// The identifier.
    identifier: Identifier,
    /// The source of the chunk data.
    data: DataSource<'a>,
}

impl<'a> ChunkDesc<'a> {
    /// Create a new chunk desc.
    pub fn new(identifier: Identifier, data: DataSource<'a>) -> Self {
        Self { identifier, data }
    }

    /// Get the chunk identifier.
    pub fn identifier(&self) -> Identifier {
        self.identifier
    }

    /// Take chunk and return the data source.
    pub fn data_source(self) -> DataSource<'a> {
        self.data
    }
}
