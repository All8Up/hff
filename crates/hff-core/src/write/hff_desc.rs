use super::{ChunkArray, DataArray, TableArray};
use crate::{Chunk, Header, Table};

/// Description of hff and content.
#[derive(Debug)]
pub struct HffDesc<'a> {
    /// The tables.
    tables: TableArray,
    /// The chunks.
    chunks: ChunkArray,
    /// The data blob.
    data: Option<DataArray<'a>>,
}

impl<'a> HffDesc<'a> {
    /// Create a new content instance.
    pub fn new(tables: TableArray, chunks: ChunkArray, data: DataArray<'a>) -> Self {
        Self {
            tables,
            chunks,
            data: Some(data),
        }
    }

    /// Finish the descriptor and return the component parts.
    pub fn finish(self) -> (TableArray, ChunkArray, DataArray<'a>) {
        (self.tables, self.chunks, self.data.unwrap())
    }

    /// Update tables and chunks for the given offset and length data.
    pub fn update_data(
        tables: &mut TableArray,
        chunks: &mut ChunkArray,
        offset: u64,
        offset_len: &[(u64, u64)],
    ) {
        let mut table_index = 0;
        let mut chunk_index = 0;
        let mut chunk_count = 0;
        let mut entry = 0;

        loop {
            if chunk_count > 0 {
                // We are filling in chunks.
                *chunks[chunk_index].offset_mut() = offset_len[entry].0 + offset;
                *chunks[chunk_index].length_mut() = offset_len[entry].1;

                chunk_count -= 1;
                entry += 1;
                chunk_index += 1;
                continue;
            }

            if table_index == tables.len() {
                break;
            }

            if tables[table_index].0 {
                // We have metadata in this table.
                *tables[table_index].1.metadata_offset_mut() = offset_len[entry].0 + offset;
                *tables[table_index].1.metadata_length_mut() = offset_len[entry].1;
                entry += 1;
            }

            chunk_count = tables[table_index].1.chunk_count();
            table_index += 1;
        }
    }
    /// Size of the content arrays.
    pub fn arrays_size(&self) -> usize {
        self.tables.len() * Table::SIZE + self.chunks.len() * Chunk::SIZE
    }

    /// Calculate the offset from the start of the file to the start of
    /// the data blob.
    pub fn offset_to_blob(&self) -> usize {
        Header::SIZE + self.arrays_size()
    }
}
