use super::{ChunkReader, ReadSeek};
use hff_core::{read::ChunkView, Result};

/// Extension to read a chunk from a table.
#[async_trait::async_trait]
pub trait Chunk {
    /// Read the metadata from the table.
    async fn read<T: ReadSeek>(&self, source: &mut ChunkReader<T>) -> Result<Vec<u8>>;
}

#[async_trait::async_trait]
impl<'a> Chunk for ChunkView<'a> {
    async fn read<T: ReadSeek>(&self, source: &mut ChunkReader<T>) -> Result<Vec<u8>> {
        let chunk = &self.hff().chunks_array()[self.index()];
        if chunk.length() > 0 {
            let mut buffer = vec![0; chunk.length() as usize];
            source.read(chunk.offset(), buffer.as_mut_slice()).await?;
            Ok(buffer)
        } else {
            Ok(vec![])
        }
    }
}
