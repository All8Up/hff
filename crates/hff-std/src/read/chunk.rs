use crate::{ReadSeek, Result};
use hff_core::read::ChunkView;
use std::io::SeekFrom;

/// Extension to read metadata from a table.
pub trait Chunk {
    /// Read the chunk data from the table.
    fn read(&self, source: &mut dyn ReadSeek) -> Result<Vec<u8>>;

    /// Read the chunk data into the provided buffer.
    fn read_exact(&self, source: &mut dyn ReadSeek, buffer: &mut [u8]) -> Result<u64>;
}

impl<'a> Chunk for ChunkView<'a> {
    fn read(&self, source: &mut dyn ReadSeek) -> Result<Vec<u8>> {
        let chunk = &self.hff().chunks_array()[self.index()];
        if chunk.length() > 0 {
            source.seek(SeekFrom::Start(chunk.offset()))?;
            let mut buffer = vec![0; chunk.length() as usize];
            source.read_exact(buffer.as_mut_slice())?;
            Ok(buffer)
        } else {
            Ok(vec![])
        }
    }

    /// Read the chunk data into the provided buffer.
    fn read_exact(&self, source: &mut dyn ReadSeek, buffer: &mut [u8]) -> Result<u64> {
        let chunk = &self.hff().chunks_array()[self.index()];
        if chunk.length() > 0 {
            source.seek(SeekFrom::Start(chunk.offset()))?;
            source.read_exact(buffer)?;
            Ok(buffer.len() as u64)
        } else {
            Ok(0)
        }
    }
}
