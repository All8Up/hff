use std::{
    fmt::Debug,
    io::{Cursor, Read, Seek, SeekFrom},
};

/// Act as a ReadSeek IO object for purposes of having
/// an entire HFF in memory at one time.
#[derive(Debug, Clone)]
pub struct ChunkCache {
    offset: u64,
    buffer: Cursor<Vec<u8>>,
}

impl ChunkCache {
    /// Create a new chunk cache.
    pub fn new(offset: usize, buffer: Vec<u8>) -> Self {
        Self {
            offset: offset as u64,
            buffer: Cursor::new(buffer),
        }
    }
}

impl Read for ChunkCache {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.buffer.read(buf)
    }
}

impl Seek for ChunkCache {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        // Adjust the position if it is from the start because we want
        // to act as if we are in the file 'after' the header+tables.
        let pos = match pos {
            SeekFrom::Current(p) => SeekFrom::Current(p),
            SeekFrom::Start(p) => SeekFrom::Start(p - self.offset),
            SeekFrom::End(p) => SeekFrom::End(p),
        };
        self.buffer.seek(pos)
    }
}
