use async_std::io::prelude::*;
use hff_core::Result;

pub trait ReadSeek: Read + Seek + Send + Sync + Unpin {}
impl<T: Read + Seek + Send + Sync + Unpin> ReadSeek for T {}

pub struct ChunkReader<T: ReadSeek> {
    read_seek: T,
}

impl<T: ReadSeek> ChunkReader<T> {
    /// Create a new chunk reader.
    pub fn new(read_seek: T) -> Self {
        Self { read_seek }
    }

    /// Read a chunk of data from the contained type.
    pub async fn read(&mut self, position: u64, buffer: &mut [u8]) -> Result<()> {
        self.read_seek
            .seek(std::io::SeekFrom::Start(position))
            .await?;
        self.read_seek.read_exact(buffer).await?;
        Ok(())
    }
}
