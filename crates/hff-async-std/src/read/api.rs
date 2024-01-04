use super::{AsyncStdReader, ReadSeek};
use async_std::io::Read;
use hff_core::{
    read::{Hff, Inspection},
    ChunkCache, Result, NE, OP,
};

/// Opens the input and maintains it for random access to the
/// metadata and chunks.
pub async fn open(mut source: impl ReadSeek + 'static) -> Result<Hff<AsyncStdReader>> {
    let header = AsyncStdReader::read_header(&mut source).await?;
    let (tables, chunks) = if header.is_native_endian() {
        (
            AsyncStdReader::read_tables::<NE>(&mut source, header.table_count()).await?,
            AsyncStdReader::read_chunks::<NE>(&mut source, header.chunk_count()).await?,
        )
    } else {
        (
            AsyncStdReader::read_tables::<OP>(&mut source, header.table_count()).await?,
            AsyncStdReader::read_chunks::<OP>(&mut source, header.chunk_count()).await?,
        )
    };
    Ok(Hff::new(
        AsyncStdReader::new(source),
        header,
        tables,
        chunks,
    ))
}

/// Reads an entire Hff into memory.
pub async fn read(mut source: &mut (dyn Read + std::marker::Unpin)) -> Result<Hff<ChunkCache>> {
    let header = AsyncStdReader::read_header(&mut source).await?;
    let (tables, chunks) = if header.is_native_endian() {
        (
            AsyncStdReader::read_tables::<NE>(&mut source, header.table_count()).await?,
            AsyncStdReader::read_chunks::<NE>(&mut source, header.chunk_count()).await?,
        )
    } else {
        (
            AsyncStdReader::read_tables::<OP>(&mut source, header.table_count()).await?,
            AsyncStdReader::read_chunks::<OP>(&mut source, header.chunk_count()).await?,
        )
    };
    let cache = AsyncStdReader::read_body(&mut source, tables.len(), chunks.len()).await?;
    Ok(Hff::new(cache, header, tables, chunks))
}

/// Read the structure of a Hff into memory.  Provides access
/// only to the structure without any of the metadata or chunk
/// data available.
pub async fn inspect(mut source: &mut (dyn Read + std::marker::Unpin)) -> Result<Hff<Inspection>> {
    let header = AsyncStdReader::read_header(&mut source).await?;
    let (tables, chunks) = if header.is_native_endian() {
        (
            AsyncStdReader::read_tables::<NE>(&mut source, header.table_count()).await?,
            AsyncStdReader::read_chunks::<NE>(&mut source, header.chunk_count()).await?,
        )
    } else {
        (
            AsyncStdReader::read_tables::<OP>(&mut source, header.table_count()).await?,
            AsyncStdReader::read_chunks::<OP>(&mut source, header.chunk_count()).await?,
        )
    };
    Ok(Hff::new(Inspection, header, tables, chunks))
}
