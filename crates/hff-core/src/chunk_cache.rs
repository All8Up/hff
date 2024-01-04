use crate::{ContentInfo, Error, Result};

/// Act as a ReadSeek IO object for purposes of having
/// an entire HFF in memory at one time.
#[derive(Debug, Clone)]
pub struct ChunkCache {
    offset: u64,
    buffer: Vec<u8>,
}

impl ChunkCache {
    /// Create a new chunk cache.
    pub fn new(offset: usize, buffer: Vec<u8>) -> Self {
        Self {
            offset: offset as u64,
            buffer,
        }
    }

    /// Get a slice representing the given content.
    pub fn read(&self, content: &dyn ContentInfo) -> Result<&'_ [u8]> {
        if content.len() > 0 {
            assert!(
                content.offset() >= self.offset,
                "{} {}",
                content.offset(),
                self.offset
            );
            let offset = content.offset() - self.offset;
            Ok(&self.buffer[offset as usize..(offset + content.len()) as usize])
        } else {
            Err(Error::Invalid("No data for this content.".into()))
        }
    }
}
