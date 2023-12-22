use crate::Result;
use hff_core::{ByteOrder, Chunk};
use std::{
    io::Write,
    ops::{Index, IndexMut},
};

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

    /// Write the tables to the given stream.
    pub fn write<E: ByteOrder>(self, writer: &mut dyn Write) -> Result<()> {
        for chunk in self.chunks {
            chunk.write::<E>(writer)?;
        }
        Ok(())
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
