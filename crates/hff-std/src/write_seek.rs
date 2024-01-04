use std::io::{Seek, Write};

/// Helper trait for lazy writing.
pub trait WriteSeek: Write + Seek {}

/// Blanket implementation for anything viable.
impl<T: Write + Seek> WriteSeek for T {}
