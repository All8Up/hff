use std::io::{Read, Seek};

/// A wrapper trait with a blanket implementation for any type which
/// supports both Read and Seek.  The data source of the Hff must
/// support this in order to retrieve metadata or chunk's from the
/// stream.  If the stream is not Seek, either use the provided
/// `_full` variation which will return the ChunkCache or otherwise
/// store the entire file somewhere which can be accessed with Seek.
pub trait ReadSeek: Read + Seek {}

// Implement over anything which can read and seek.
impl<T: Read + Seek> ReadSeek for T {}
