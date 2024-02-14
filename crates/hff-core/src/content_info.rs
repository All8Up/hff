/// Information about the metadata or chunk data contained within the source.
pub trait ContentInfo {
    /// Length of the data.
    fn len(&self) -> u64;
    /// Offset into the overall source.
    fn offset(&self) -> u64;
}

// Helper to use content info from provided data.
// Not all ways of access the chunks will be through the views.
impl ContentInfo for (u64, u64) {
    fn len(&self) -> u64 {
        self.0
    }

    fn offset(&self) -> u64 {
        self.1
    }
}
