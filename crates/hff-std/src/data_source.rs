use super::{Error, Result};
use std::{fmt::Debug, io::Write};

/// A chunk data source.
pub trait DataSource: Debug {
    /// Length of the data if available.
    fn len(&self) -> Option<u64>;

    /// Prepare the chunk data if the length was not available.
    /// Returns the size of the data that will be stored.
    fn prepare(&mut self) -> Result<u64>;

    /// Get the underlying data type.
    fn as_datatype(&mut self) -> &mut dyn std::any::Any;
}

/// Extends the datasource to be able to write into a std::io::Write.
pub trait StdWriter: DataSource {
    /// Write the data to the given stream.
    fn write(&mut self, writer: &mut dyn Write) -> Result<()>;
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

    fn as_datatype(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl StdWriter for OwnedDataSource {
    fn write(&mut self, writer: &mut dyn Write) -> Result<()> {
        std::io::copy(&mut self.0.as_slice(), writer)?;
        Ok(())
    }
}

impl TryInto<Box<dyn DataSource>> for &str {
    type Error = Error;

    fn try_into(self) -> Result<Box<dyn DataSource>> {
        Ok(Box::new(OwnedDataSource(self.as_bytes().into())))
    }
}

impl<'a> TryInto<&'a mut dyn StdWriter> for &'a mut Box<dyn DataSource> {
    type Error = Error;

    fn try_into(self) -> Result<&'a mut dyn StdWriter> {
        let data_type = self.as_datatype();

        // TODO: There is likely some way to do this in a cleaner way but this will
        // work for the moment.
        let owned = data_type.downcast_mut::<OwnedDataSource>();
        if let Some(owned) = owned {
            Ok(owned)
        } else {
            Err(Error::Invalid("Unknown data type.".into()))
        }
    }
}
