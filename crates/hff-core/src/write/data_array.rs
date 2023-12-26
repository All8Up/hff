use super::DataSource;
use crate::Result;
use std::ops::{Deref, DerefMut};

/// Storage of data sources for writing the hff content.
#[derive(Debug)]
pub struct DataArray<'a> {
    /// The vector of data sources.
    data: Vec<DataSource<'a>>,
}

impl<'a> DataArray<'a> {
    /// Create a new empty data array.
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    /// Push a new data source onto the array.
    pub fn push(&mut self, item: DataSource<'a>) {
        self.data.push(item);
    }

    /// Prepare the data in the array.
    pub fn prepare(&mut self) -> Result<Vec<(u64, u64)>> {
        let mut offset_len = vec![];
        let mut offset = 0;
        for entry in &mut self.data {
            let length = if let Some(length) = entry.len() {
                length as u64
            } else {
                entry.prepare()?
            };
            offset_len.push((offset, length));

            let padding = length.next_multiple_of(16) - length;
            offset += length + padding;
        }

        Ok(offset_len)
    }
}

impl<'a> IntoIterator for DataArray<'a> {
    type Item = DataSource<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> Deref for DataArray<'a> {
    type Target = [DataSource<'a>];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a> DerefMut for DataArray<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
