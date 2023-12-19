use super::{Error, Result};
use std::fmt::Debug;

/// A chunk data source.
pub trait DataSource: Debug {
    /// Length of the data if available.
    fn len(&self) -> Option<u64>;

    /// Prepare the chunk data if the length was not available.
    /// Returns the size of the data that will be stored.
    fn prepare(&mut self) -> Result<u64>;
}

// Internal structure to deal with raw data sources such as
// vectors, strings etc that are directly available in memory.
struct OwnedDataSource(Vec<u8>);

impl Debug for OwnedDataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<OwnedDataSource>")
    }
}

impl DataSource for OwnedDataSource {
    fn len(&self) -> Option<u64> {
        Some(self.0.len() as u64)
    }

    fn prepare(&mut self) -> Result<u64> {
        Ok(self.0.len() as u64)
    }
}

impl TryInto<Box<dyn DataSource>> for &str {
    type Error = Error;

    fn try_into(self) -> Result<Box<dyn DataSource>> {
        Ok(Box::new(OwnedDataSource(self.as_bytes().into())))
    }
}
