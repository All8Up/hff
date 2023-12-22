use super::{ChunkArray, DataArray, TableArray};
use crate::Result;
use hff_core::{ByteOrder, Chunk, Ecc, Header, Table};
use std::io::{Seek, Write};

/// Content to be written into an hff stream.
#[derive(Debug)]
pub struct HffContent {
    /// The tables.
    tables: TableArray,
    /// The chunks.
    chunks: ChunkArray,
    /// The data blob.
    data: DataArray,
}

impl HffContent {
    /// Create a new content instance.
    pub fn new(tables: TableArray, chunks: ChunkArray, data: DataArray) -> Self {
        Self {
            tables,
            chunks,
            data,
        }
    }

    /// Write the header to the given stream.
    pub fn write_header<E: ByteOrder>(
        &self,
        content_type: Ecc,
        writer: &mut dyn Write,
    ) -> Result<()> {
        let header = Header::new(
            content_type,
            self.tables.len() as u32,
            self.chunks.len() as u32,
        );
        header.write::<E>(writer)?;

        Ok(())
    }

    /// Update tables and chunks for the given offset and length data.
    fn update_data(&mut self, offset: u64, offset_len: &[(u64, u64)]) {
        let mut table_index = 0;
        let mut chunk_index = 0;
        let mut chunk_count = 0;
        let mut entry = 0;

        loop {
            if chunk_count > 0 {
                // We are filling in chunks.
                *self.chunks[chunk_index].offset_mut() = offset_len[entry].0 + offset;
                *self.chunks[chunk_index].length_mut() = offset_len[entry].1;

                chunk_count -= 1;
                entry += 1;
                chunk_index += 1;
                continue;
            }

            if table_index == self.tables.len() {
                break;
            }

            if self.tables[table_index].0 {
                // We have metadata in this table.
                *self.tables[table_index].1.metadata_offset_mut() = offset_len[entry].0 + offset;
                *self.tables[table_index].1.metadata_length_mut() = offset_len[entry].1;
                entry += 1;
            }

            chunk_count = self.tables[table_index].1.chunk_count();
            table_index += 1;
        }
    }

    /// Calculate the offset from the start of the file to the start of
    /// the data blob.
    pub fn offset_to_blob(&self) -> usize {
        Header::SIZE + self.tables.len() * Table::SIZE + self.chunks.len() * Chunk::SIZE
    }

    /// Write the content to the given stream without seek abilities.
    pub fn write<E: ByteOrder>(
        mut self,
        content_type: impl Into<Ecc>,
        writer: &mut dyn Write,
    ) -> Result<()> {
        self.write_header::<E>(content_type.into(), writer)?;

        // Prepare all the data in the data array so we have offsets and length.
        let offset_len = self.data.prepare()?;
        // Compute the size of the header so we can offset the data blob information.
        let header_offset = self.offset_to_blob();

        // Update the table metadata length/offset and chunk length/offset.
        self.update_data(header_offset as u64, &offset_len);

        self.tables.write::<E>(writer)?;
        self.chunks.write::<E>(writer)?;
        self.data.write(writer)?;

        Ok(())
    }

    /// Write the content to the given stream.  This requires seek because we
    /// update the table and chunk entries 'after' writing the data blob and as
    /// such, we have to go back and write them.
    pub fn lazy_write<E: ByteOrder, W: Write + Seek>(
        self,
        content_type: impl Into<Ecc>,
        writer: &mut W,
    ) -> Result<()> {
        self.write_header::<E>(content_type.into(), writer)?;

        unimplemented!("TODO.");

        Ok(())
    }
}
