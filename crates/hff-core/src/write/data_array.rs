use super::DataSource;
use crate::Result;
use std::{
    io::Write,
    ops::{Deref, DerefMut},
};

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

    /// Write the data to the given stream.
    /// Returns a vector of offset into the writer (starting from 0)
    /// and the length of the data written without alignment padding.
    pub fn write(self, writer: &mut dyn Write) -> Result<Vec<(u64, u64)>> {
        let mut offset_len = vec![];

        // Track where we are in the writer, starting from zero.
        let mut offset = 0;
        for mut item in self.data {
            // Prepare each item.
            // This is only for compressed data (at this time) to perform
            // the compression.  Using std write here means it all has to
            // be buffered into memory.
            item.prepare()?;

            // Write in the appropriate manner.
            let length = match item {
                DataSource::File(mut f, _) => std::io::copy(&mut f, writer)?,
                DataSource::Owned(data) => std::io::copy(&mut data.as_slice(), writer)?,
                DataSource::Ref(mut data) => std::io::copy(&mut data, writer)?,
                #[cfg(feature = "compression")]
                DataSource::Compressed(_, _, data) => {
                    std::io::copy(&mut data.unwrap().as_slice(), writer)?
                }
            };

            // Record the offset and length.
            offset_len.push((offset as u64, length));

            // What is the padding requirement?
            let padding = (length.next_multiple_of(16) - length) as usize;
            // Track where we are in the output stream.
            offset += length as usize + padding;

            // Write the padding.
            let padding = vec![0; padding];
            writer.write_all(&padding)?;
        }

        Ok(offset_len)
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
