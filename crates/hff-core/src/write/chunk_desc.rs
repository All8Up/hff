use super::DataSource;
use crate::Ecc;

/// An intermediate chunk description.
#[derive(Debug)]
pub struct ChunkDesc<'a> {
    /// The primary identifier.
    primary: Ecc,
    /// The secondary identifier.
    secondary: Ecc,
    /// The source of the chunk data.
    data: DataSource<'a>,
}

impl<'a> ChunkDesc<'a> {
    /// Create a new chunk desc.
    pub fn new(primary: Ecc, secondary: Ecc, data: DataSource<'a>) -> Self {
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

    /// Take chunk and return the data source.
    pub fn data_source(self) -> DataSource<'a> {
        self.data
    }
}
