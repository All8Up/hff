use super::{ChunkArray, DataArray, TableArray};
use crate::{ByteOrder, Chunk, Ecc, Header, Result, Table};
use std::io::{Seek, Write};

/// Helper trait for lazy writing.
pub trait WriteSeek: Write + Seek {}

/// Blanket implementation for anything viable.
impl<T: Write + Seek> WriteSeek for T {}

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

    /// Size of the content arrays.
    pub fn arrays_size(&self) -> usize {
        self.tables.len() * Table::SIZE + self.chunks.len() * Chunk::SIZE
    }

    /// Calculate the offset from the start of the file to the start of
    /// the data blob.
    pub fn offset_to_blob(&self) -> usize {
        Header::SIZE + self.arrays_size()
    }

    /// Write the content to the given stream without seek abilities.
    pub fn write<E: ByteOrder>(
        mut self,
        content_type: impl Into<Ecc>,
        writer: &mut dyn Write,
    ) -> Result<()> {
        self.write_header::<E>(content_type.into(), writer)?;

        // Prepare all the data in the data array so we have offsets and length.
        let mut data = self.data.take().unwrap();
        let offset_len = data.prepare()?;

        // Update the table metadata length/offset and chunk length/offset.
        self.update_data(self.offset_to_blob() as u64, &offset_len);

        writer.write_all(self.tables.to_bytes::<E>()?.as_slice())?;
        writer.write_all(self.chunks.to_bytes::<E>()?.as_slice())?;
        let _test = data.write(writer)?;
        assert_eq!(_test, offset_len);

        Ok(())
    }

    /// Write the content to the given stream.  This requires seek because we
    /// update the table and chunk entries 'after' writing the data blob and as
    /// such, we have to go back and write them.
    pub fn lazy_write<E: ByteOrder>(
        mut self,
        content_type: impl Into<Ecc>,
        mut writer: &mut dyn WriteSeek,
    ) -> Result<()> {
        self.write_header::<E>(content_type.into(), &mut writer)?;

        // Write zero's for the table and chunk array.
        // Use this rather than skipping in order to avoid any questionable
        // differences between different backing types.
        writer.write_all(&mut vec![0; self.arrays_size()])?;

        // Write the data and record the offset/length information.
        let data = self.data.take().unwrap();
        let offset_len = data.write(&mut writer)?;

        // Update the table metadata length/offset and chunk length/offset.
        self.update_data(self.offset_to_blob() as u64, &offset_len);

        // Seek back to the tables/chunks.
        writer.seek(std::io::SeekFrom::Start(Header::SIZE as u64))?;

        // And write the tables and chunks.
        writer.write_all(self.tables.to_bytes::<E>()?.as_slice())?;
        writer.write_all(self.chunks.to_bytes::<E>()?.as_slice())?;

        Ok(())
    }
}
