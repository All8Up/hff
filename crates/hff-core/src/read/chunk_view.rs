#[cfg(feature = "compression")]
use crate::Result;
use crate::{read::Hff, Ecc};

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

    /// Get the hff this was built from.
    pub fn hff(&self) -> &Hff {
        self.hff
    }

    /// Get the current index.
    pub fn index(&self) -> usize {
        self.index
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

    /// Decompress the provided data.
    #[cfg(feature = "compression")]
    pub fn decompress(source: &[u8]) -> Result<Vec<u8>> {
        if source.len() > 0 {
            let mut decoder = xz2::read::XzDecoder::new(source);
            let mut result = vec![];
            std::io::copy(&mut decoder, &mut result)?;
            Ok(result)
        } else {
            Ok(vec![])
        }
    }
}
