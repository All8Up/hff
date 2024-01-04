/// Information about the metadata or chunk data contained within the source.
pub trait ContentInfo {
    /// Length of the data.
    fn len(&self) -> u64;
    /// Offset into the overall source.
    fn offset(&self) -> u64;
}
