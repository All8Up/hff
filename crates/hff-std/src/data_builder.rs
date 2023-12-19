use std::fmt::Debug;

use super::DataSource;

/// Helper to track and build chunks.
pub struct DataBuilder {
    /// Chunks.
    chunks: Vec<(Box<dyn DataSource>, u64)>,
}

impl Debug for DataBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        for (index, chunk) in self.chunks.iter().enumerate() {
            list.entry(&format!("{}: {:?}", index, chunk));
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
}
