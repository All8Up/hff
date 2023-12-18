use super::Result;
use std::{fmt::Debug, io::Read};

/// A chunk data source.
pub trait DataSource: Debug {
    /// Length of the data if available.
    fn len(&self) -> Option<u64>;

    /// Prepare the chunk data if the length was not available
    /// and there is no other way to find the length.  Generally
    /// this happens for something like a compressed chunk which
    /// we won't know the final size of until it is compressed.
    /// Returns the size after preparation.
    fn prepare(&mut self) -> Result<u64>;
}
