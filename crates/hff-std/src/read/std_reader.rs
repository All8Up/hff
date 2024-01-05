use crate::ReadSeek;
use hff_core::{ContentInfo, Error, Result};

/// Implements a std reader wrapper around the source.
pub struct StdReader {
    source: std::sync::Mutex<Box<dyn ReadSeek>>,
}

impl std::fmt::Debug for StdReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReadSeek")
    }
}

impl StdReader {
    /// Create a new std reader type.
    pub fn new(source: impl ReadSeek + 'static) -> Self {
        Self {
            source: std::sync::Mutex::new(Box::new(source)),
        }
    }

    /// Get the content of the given item.
    pub fn get(&self, content: &dyn ContentInfo) -> Result<Vec<u8>> {
        let mut source = self
            .source
            .lock()
            .map_err(|e| Error::Invalid(e.to_string()))?;
        source.seek(std::io::SeekFrom::Start(content.offset()))?;

        let mut result = vec![0; content.len() as usize];
        source.read_exact(&mut result)?;
        Ok(result)
    }

    /// Read the content into the provided slice.
    pub fn read_exact(&self, content: &dyn ContentInfo, buffer: &mut [u8]) -> Result<()> {
        let mut source = self
            .source
            .lock()
            .map_err(|e| Error::Invalid(e.to_string()))?;
        source.seek(std::io::SeekFrom::Start(content.offset()))?;

        source.read_exact(buffer)?;
        Ok(())
    }

    /// Get the slice of data representing the content requested.
    pub fn read(
        &self,
        content: &dyn ContentInfo,
    ) -> Result<std::sync::MutexGuard<'_, Box<dyn ReadSeek>>> {
        let mut source = self
            .source
            .lock()
            .map_err(|e| Error::Invalid(e.to_string()))?;
        source.seek(std::io::SeekFrom::Start(content.offset()))?;
        Ok(source)
    }
}
