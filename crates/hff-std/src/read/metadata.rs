use crate::{ReadSeek, Result};
use hff_core::TableView;
use std::io::SeekFrom;

/// Extension to read metadata from a table.
pub trait Metadata {
    /// Read the metadata from the table.
    fn metadata(&self, source: &mut dyn ReadSeek) -> Result<Vec<u8>>;
}

impl<'a> Metadata for TableView<'a> {
    fn metadata(&self, source: &mut dyn ReadSeek) -> Result<Vec<u8>> {
        let table = &self.hff().tables_array()[self.index()];
        if table.metadata_length() > 0 {
            source.seek(SeekFrom::Start(table.metadata_offset()))?;
            let mut buffer = vec![0; table.metadata_length() as usize];
            source.read_exact(buffer.as_mut_slice())?;
            Ok(buffer)
        } else {
            Ok(vec![])
        }
    }
}
