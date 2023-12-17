use hff_core::Ecc;

/// Internal description of a chunk and the data source for the contents.
#[derive(Debug, Clone)]
pub struct ChunkDesc {
    /// The primary identifier for this chunk.
    primary: Ecc,
    /// The secondary identifier for this chunk.
    secondary: Ecc,
    /// The boxed chunk data source.
    data: (),
}
