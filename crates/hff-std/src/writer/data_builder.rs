use crate::{DataSource, Result, StdWriter};
use std::fmt::Debug;

/// Helper to track and build chunks.
pub struct DataBuilder {
    /// Chunks.
    chunks: Vec<(Box<dyn DataSource>, u64)>,
}

impl Debug for DataBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        let mut offset = 0;
        for (index, chunk) in self.chunks.iter().enumerate() {
            list.entry(&format!("{}: {} {:?}", index, offset, chunk));
            offset += chunk.1.next_multiple_of(16);
        }
        list.finish()
    }
}

impl DataBuilder {
    /// Create an empty instance.
    pub fn new() -> Self {
        Self { chunks: vec![] }
    }

    /// Push a chunk onto the builder.
    pub fn push(&mut self, data_source: Box<dyn DataSource>, size: u64) {
        self.chunks.push((data_source, size));
    }

    /// Append a builder to this builder.
    pub fn append(&mut self, mut child: Self) {
        self.chunks.append(&mut child.chunks);
    }

    /// Get the size in bytes of the contained data.
    pub fn size_bytes(&self) -> u64 {
        let mut count = 0;
        for chunk in &self.chunks {
            let length = chunk.1.next_multiple_of(16);
            count += length;
        }

        count
    }

    /// Write the data to the given writer.
    pub fn write(self, writer: &mut dyn std::io::Write) -> Result<()> {
        for mut chunk in self.chunks {
            let padding = chunk.1.next_multiple_of(16) - chunk.1;

            // TODO: There is probably some nice way to do this but it is eluding me at the moment.
            let std_writer: &mut dyn StdWriter = (&mut chunk.0).try_into()?;
            // Write the chunk data.
            std_writer.write(writer)?;

            // Write the padding.
            writer.write_all(&vec![0; padding as usize])?;
        }

        Ok(())
    }
}
