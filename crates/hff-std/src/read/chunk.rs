use crate::{ReadSeek, Result};
use hff_core::ChunkView;
use std::io::SeekFrom;

/// Extension to read metadata from a table.
pub trait Chunk {
    /// Read the chunk data from the table.
    fn read(&self, source: &mut dyn ReadSeek) -> Result<Vec<u8>>;

    /// Read and decompress a chunk from the table.
    #[cfg(feature = "compression")]
    fn decompress(&self, source: &mut dyn ReadSeek) -> Result<Vec<u8>>;
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

    /// Read and decompress the chunk data.
    #[cfg(feature = "compression")]
    fn decompress(&self, source: &mut dyn ReadSeek) -> Result<Vec<u8>> {
        let source = self.read(source)?;
        if source.len() > 0 {
            let source: &mut dyn std::io::Read = &mut source.as_slice();
            let mut decoder = xz2::read::XzDecoder::new(source);
            let mut result = vec![];
            std::io::copy(&mut decoder, &mut result)?;
            Ok(result)
        } else {
            Ok(vec![])
        }
    }
}
