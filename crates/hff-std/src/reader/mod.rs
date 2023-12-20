use hff_core::{ByteOrder, Chunk, Header, Result, Table, NE, OP};
use std::{io::Read, mem::size_of};

mod read_seek;
pub use read_seek::ReadSeek;

mod hff_cache;
pub use hff_cache::ChunkCache;

mod hff;
pub use hff::Hff;

mod table_view;
pub use table_view::TableView;

mod chunk_view;
pub use chunk_view::ChunkView;

mod chunk_iter;
pub use chunk_iter::ChunkIter;

mod table_iter;
pub use table_iter::TableIter;

mod depth_first_iter;
pub use depth_first_iter::DepthFirstIter;

/// Read a HFF from the given stream.
pub fn read_stream(reader: &mut dyn Read) -> Result<Hff> {
    // The header determines the structure endianess.
    let header = Header::read(reader)?;
    if header.is_native_endian() {
        let tables = read_tables::<NE>(reader, header.table_count())?;
        let chunks = read_chunks::<NE>(reader, header.chunk_count())?;
        Ok(Hff::new(tables, chunks))
    } else {
        let tables = read_tables::<OP>(reader, header.table_count())?;
        let chunks = read_chunks::<OP>(reader, header.chunk_count())?;
        Ok(Hff::new(tables, chunks))
    }
}

/// Read a HFF from the given stream along with all the data for the chunks.
pub fn read_stream_full(reader: &mut dyn Read) -> Result<(Hff, ChunkCache)> {
    let hff = read_stream(reader)?;
    let mut buffer = vec![];
    reader.read_to_end(&mut buffer)?;

    let offset = hff.offset_to_data();
    Ok((hff, ChunkCache::new(offset, buffer)))
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
