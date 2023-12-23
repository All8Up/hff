use crate::DataSource;
use hff_core::Ecc;

#[derive(Debug)]
pub struct ChunkDesc<'a> {
    primary: Ecc,
    secondary: Ecc,
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
