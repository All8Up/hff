use crate::{Hff, ReadSeek};
use hff_core::{Ecc, Result};
use std::io::SeekFrom;

/// A view to a chunk.
pub struct ChunkView<'a> {
    hff: &'a Hff,
    index: usize,
}

impl<'a> ChunkView<'a> {
    /// Create a new view.
    pub fn new(hff: &'a Hff, index: usize) -> Self {
        Self { hff, index }
    }

    /// Get the primary identifier.
    pub fn primary(&self) -> Ecc {
        self.hff.chunks_array()[self.index].primary()
    }

    /// Get the secondary identifier.
    pub fn secondary(&self) -> Ecc {
        self.hff.chunks_array()[self.index].secondary()
    }

    /// Get the size of the data in the chunk.
    pub fn size(&self) -> usize {
        self.hff.chunks_array()[self.index].length() as usize
    }

    /// Get the data for the chunk from a read/seek source.
    pub fn data(&self, source: &mut dyn ReadSeek) -> Result<Vec<u8>> {
        let chunk = &self.hff.chunks_array()[self.index];
        if chunk.length() > 0 {
            source.seek(SeekFrom::Start(chunk.offset()))?;
            let mut buffer = vec![0; chunk.length() as usize];
            source.read_exact(buffer.as_mut_slice())?;
            Ok(buffer)
        } else {
            Ok(vec![])
        }
    }
}
