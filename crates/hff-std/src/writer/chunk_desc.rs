use crate::{DataSource, Error, Result};
use hff_core::{Chunk, Ecc, Table};

#[derive(Debug)]
pub struct ChunkDesc {
    primary: Ecc,
    secondary: Ecc,
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

    /// Take chunk and return the data source.
    pub fn data_source(self) -> Box<dyn DataSource> {
        self.data
    }
}
