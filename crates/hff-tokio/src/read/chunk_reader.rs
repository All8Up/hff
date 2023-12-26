use hff_core::Result;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt};

pub trait ReadSeek: AsyncRead + AsyncSeek + Send + Sync + Unpin {}
impl<T: AsyncRead + AsyncSeek + Send + Sync + Unpin> ReadSeek for T {}

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
