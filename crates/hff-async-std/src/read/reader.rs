use super::ChunkCache;
use async_std::io::prelude::*; //{Read, ReadExt};
use hff_core::{read::Hff, ByteOrder, Chunk, Header, Result, Table, NE, OP};
use std::mem::size_of;

/// An extension to read in Hff files using std::io.
#[async_trait::async_trait]
pub trait Reader {
    /// Read in the Hff.  Only reads the structure of the Hff.
    async fn read<T: Read + Unpin + Sync + Send>(reader: &mut T) -> Result<Hff>;

    /// Read in the full Hff, structure and chunks.
    async fn read_full<T: Read + Unpin + Sync + Send>(reader: &mut T) -> Result<(Hff, ChunkCache)>;
}

#[async_trait::async_trait]
impl Reader for Hff {
    async fn read<T: Read + Unpin + Sync + Send>(reader: &mut T) -> Result<Hff> {
        // The header determines the structure endianess.
        let header = read_header(reader).await?;
        let (tables, chunks) = if header.is_native_endian() {
            (
                read_tables::<NE, T>(reader, header.table_count()).await?,
                read_chunks::<NE, T>(reader, header.chunk_count()).await?,
            )
        } else {
            (
                read_tables::<OP, T>(reader, header.table_count()).await?,
                read_chunks::<OP, T>(reader, header.chunk_count()).await?,
            )
        };

        Ok(Hff::new(header, tables, chunks))
    }

    async fn read_full<T: Read + Unpin + Sync + Send>(reader: &mut T) -> Result<(Hff, ChunkCache)> {
        let hff = Self::read(reader).await?;

        let mut buffer = vec![];
        reader.read_to_end(&mut buffer).await?;

        let offset = hff.offset_to_data();
        let cache = ChunkCache::new(offset, buffer);

        Ok((hff, cache))
    }
}

/// Read the header from a given stream.
async fn read_header<T: Read + Unpin + Sync + Send>(reader: &mut T) -> Result<Header> {
    // Read the entire header worth of data.
    let mut header = vec![0; Header::SIZE];
    reader.read_exact(&mut header).await?;

    // Convert to a header.
    Ok((header.as_slice()).try_into()?)
}

async fn read_tables<E: ByteOrder, T: Read + Unpin + Sync + Send>(
    reader: &mut T,
    count: u32,
) -> Result<Vec<Table>> {
    if count > 0 {
        // Create a buffer with appropriate size.
        let mut buffer = vec![0; count as usize * size_of::<Table>()];
        reader.read_exact(&mut buffer.as_mut_slice()).await?;

        // Read all the tables out of the buffer.
        let mut tables = vec![];
        let reader: &mut dyn std::io::Read = &mut buffer.as_slice();
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

async fn read_chunks<E: ByteOrder, T: Read + Unpin + Sync + Send>(
    reader: &mut T,
    count: u32,
) -> Result<Vec<Chunk>> {
    if count > 0 {
        // Create a buffer with the appropriate size.
        let mut buffer = vec![0; count as usize * size_of::<Chunk>()];
        reader.read_exact(&mut buffer.as_mut_slice()).await?;

        // Read the chunks out of the buffer.
        let mut chunks = vec![];
        let reader: &mut dyn std::io::Read = &mut buffer.as_slice();
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
