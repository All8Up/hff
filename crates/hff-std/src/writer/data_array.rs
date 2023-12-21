use crate::{DataSource, Error, Result, StdWriter};
use hff_core::{Chunk, Ecc, Table};
use std::io::{copy, Write};

#[derive(Debug)]
pub struct DataArray {
    data: Vec<Box<dyn DataSource>>,
}

impl DataArray {
    /// Create a new empty data array.
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    /// Push a new data source onto the array.
    pub fn push(&mut self, item: Box<dyn DataSource>) {
        self.data.push(item);
    }

    /// Prepare the data in the array.
    pub fn prepare(&mut self) -> Result<Vec<(u64, u64)>> {
        let mut offset_len = vec![];
        let mut offset = 0;
        for entry in &mut self.data {
            let length = if let Some(length) = entry.len() {
                length
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
            // Convert to a std writer and write the data.
            let std_writer: &mut dyn StdWriter = (&mut item).try_into()?;
            let length = std_writer.write(writer)?;
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
