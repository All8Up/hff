use super::{ChunkDesc, TableDesc};
use crate::DataSource;
use hff_core::{Ecc, Error, Result};

/// Build a table.
#[derive(Debug)]
pub struct TableBuilder(TableDesc);

impl TableBuilder {
    /// Create a new instance.
    pub(crate) fn new(primary: Ecc, secondary: Ecc) -> Self {
        Self(TableDesc::new(primary, secondary))
    }

    /// Add a child table to the current table.
    pub fn table(mut self, content: TableDesc) -> Self {
        self.0.push_table(content);
        self
    }

    /// Add a chunk entry to the current table.
    pub fn chunk<T>(
        mut self,
        primary: impl Into<Ecc>,
        secondary: impl Into<Ecc>,
        data: T,
    ) -> Result<Self>
    where
        T: TryInto<Box<dyn DataSource>>,
        <T as TryInto<Box<dyn DataSource>>>::Error: std::fmt::Debug,
        Error: From<<T as TryInto<Box<dyn DataSource>>>::Error>,
    {
        self.0.push_chunk(ChunkDesc::new(
            primary.into(),
            secondary.into(),
            data.try_into()?,
        ));
        Ok(self)
    }

    /// Add some metadata to the current table.
    pub fn metadata<T>(mut self, data_source: T) -> Result<Self>
    where
        T: TryInto<Box<dyn DataSource>>,
        <T as TryInto<Box<dyn DataSource>>>::Error: std::fmt::Debug,
        Error: From<<T as TryInto<Box<dyn DataSource>>>::Error>,
    {
        self.0.metadata(data_source.try_into()?)?;
        Ok(self)
    }

    /// End building the table.
    pub fn end(self) -> TableDesc {
        self.0
    }
}
