use crate::{ByteOrder, Chunk, Result};
use std::ops::{Index, IndexMut};

/// After flattening a table, this is where the chunks will
/// exist.
#[derive(Debug)]
pub struct ChunkArray {
    chunks: Vec<Chunk>,
}

impl ChunkArray {
    /// Create a new empty ChunkArray.
    pub fn new() -> Self {
        Self { chunks: vec![] }
    }

    /// Get the length of the chunk array.
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// Push a new chunk onto the array.
    pub fn push(&mut self, chunk: Chunk) {
        self.chunks.push(chunk);
    }

    /// Convert the chunk array to a byte vector.
    pub fn to_bytes<E: ByteOrder>(self) -> Result<Vec<u8>> {
        let mut buffer = vec![];
        for chunk in self.chunks {
            chunk.write::<E>(&mut buffer)?;
        }
        Ok(buffer)
    }
}

impl Index<usize> for ChunkArray {
    type Output = Chunk;

    fn index(&self, index: usize) -> &Self::Output {
        &self.chunks[index]
    }
}

impl IndexMut<usize> for ChunkArray {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.chunks[index]
    }
}
