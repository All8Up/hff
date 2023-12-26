use super::{ChunkReader, ReadSeek};
use hff_core::{read::TableView, Result};

/// Extension to read metadata from a table.
#[async_trait::async_trait]
pub trait Metadata {
    /// Read the metadata from the table.
    async fn metadata<T: ReadSeek>(&self, source: &mut ChunkReader<T>) -> Result<Vec<u8>>;
}

#[async_trait::async_trait]
impl<'a> Metadata for TableView<'a> {
    async fn metadata<T: ReadSeek>(&self, source: &mut ChunkReader<T>) -> Result<Vec<u8>> {
        let table = &self.hff().tables_array()[self.index()];
        if table.metadata_length() > 0 {
            let mut buffer = vec![0; table.metadata_length() as usize];
            source
                .read(table.metadata_offset(), buffer.as_mut_slice())
                .await?;
            Ok(buffer)
        } else {
            Ok(vec![])
        }
    }
}
