use super::DataSource;
use hff_core::Ecc;

/// Internal description of a chunk and the data source for the contents.
#[derive(Debug)]
pub struct ChunkDesc {
    /// The primary identifier for this chunk.
    primary: Ecc,
    /// The secondary identifier for this chunk.
    secondary: Ecc,
    /// The boxed chunk data source.
    data: Box<dyn DataSource>,
}

impl ChunkDesc {
    /// Create a new chunk desc.
    pub fn new(primary: Ecc, secondary: Ecc, data: Box<dyn DataSource>) -> Self {
        Self {
            primary,
            secondary,
            data,
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

    /// Get the chunk data.
    pub fn data(self) -> Box<dyn DataSource> {
        self.data
    }
}
