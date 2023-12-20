use super::{Error, Result};
use std::{any::TypeId, fmt::Debug, io::Write};

mod owned;
use owned::OwnedDataSource;

mod file;
use file::FileDataSource;

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

impl<'a> TryInto<&'a mut dyn StdWriter> for &'a mut Box<dyn DataSource> {
    type Error = Error;

    fn try_into(self) -> Result<&'a mut dyn StdWriter> {
        let data_type = self.as_datatype();

        // TODO: There has to be a better way to late bind the actual
        // writer, just haven't really thought about it in detail yet
        // and this is not *TOO* horrible as long as the data source
        // foundational types don't extend too much further.
        if data_type.type_id() == TypeId::of::<OwnedDataSource>() {
            Ok(data_type.downcast_mut::<OwnedDataSource>().unwrap())
        } else if data_type.type_id() == TypeId::of::<FileDataSource>() {
            Ok(data_type.downcast_mut::<FileDataSource>().unwrap())
        } else {
            Err(Error::Invalid("Unknown data type.".into()))
        }
    }
}
