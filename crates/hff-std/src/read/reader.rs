use super::ChunkCache;
use hff_core::{read::Hff, ByteOrder, Chunk, Header, Result, Table, NE, OP};
use std::{io::Read, mem::size_of};

/// An extension to read in Hff files using std::io.
pub trait Reader {
    /// Read in the Hff.  Only reads the structure of the Hff.
    fn read(reader: &mut dyn Read) -> Result<Hff>;

    /// Read in the full Hff, structure and chunks.
    fn read_full(reader: &mut dyn Read) -> Result<(Hff, ChunkCache)>;
}

impl Reader for Hff {
    fn read(reader: &mut dyn Read) -> Result<Hff> {
        // The header determines the structure endianess.
        let header = Header::read(reader)?;
        let (tables, chunks) = if header.is_native_endian() {
            (
                read_tables::<NE>(reader, header.table_count())?,
                read_chunks::<NE>(reader, header.chunk_count())?,
            )
        } else {
            (
                read_tables::<OP>(reader, header.table_count())?,
                read_chunks::<OP>(reader, header.chunk_count())?,
            )
        };

        Ok(Hff::new(header, tables, chunks))
    }

    fn read_full(reader: &mut dyn Read) -> Result<(Hff, ChunkCache)> {
        let hff = Self::read(reader)?;

        let mut buffer = vec![];
        reader.read_to_end(&mut buffer)?;

        let offset = hff.offset_to_data();
        let cache = ChunkCache::new(offset, buffer);

        Ok((hff, cache))
    }
}

fn read_tables<E: ByteOrder>(reader: &mut dyn Read, count: u32) -> Result<Vec<Table>> {
    if count > 0 {
        // Create a buffer with appropriate size.
        let mut buffer = vec![0; count as usize * size_of::<Table>()];
        reader.read_exact(&mut buffer.as_mut_slice())?;

        // Read all the tables out of the buffer.
        let mut tables = vec![];
        let reader: &mut dyn Read = &mut buffer.as_slice();
        for _ in 0..count {
            let table = Table::read::<E>(reader)?;
            tables.push(table);
        }

        Ok(tables)
    } else {
        // TODO: Does an empty file make sense?  It's not an error but ....
        Ok(vec![])
    }
}

fn read_chunks<E: ByteOrder>(reader: &mut dyn Read, count: u32) -> Result<Vec<Chunk>> {
    if count > 0 {
        // Create a buffer with the appropriate size.
        let mut buffer = vec![0; count as usize * size_of::<Chunk>()];
        reader.read_exact(&mut buffer.as_mut_slice())?;

        // Read the chunks out of the buffer.
        let mut chunks = vec![];
        let reader: &mut dyn Read = &mut buffer.as_slice();
        for _ in 0..count {
            let chunk = Chunk::read::<E>(reader)?;
            chunks.push(chunk);
        }
        Ok(chunks)
    } else {
        // No chunks, perhaps they put all the real data into the metadata so this is
        // still a viable file.
        Ok(vec![])
    }
}
