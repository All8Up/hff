use super::{ReadSeek, TokioReader};
use hff_core::{
    read::{Hff, Inspection},
    ChunkCache, Result, NE, OP,
};
use tokio::io::AsyncRead;

/// Opens the input and maintains it for random access to the
/// metadata and chunks.
pub async fn open(mut source: impl ReadSeek + 'static) -> Result<Hff<TokioReader>> {
    let header = TokioReader::read_header(&mut source).await?;
    let (tables, chunks) = if header.is_native_endian() {
        (
            TokioReader::read_tables::<NE>(&mut source, header.table_count()).await?,
            TokioReader::read_chunks::<NE>(&mut source, header.chunk_count()).await?,
        )
    } else {
        (
            TokioReader::read_tables::<OP>(&mut source, header.table_count()).await?,
            TokioReader::read_chunks::<OP>(&mut source, header.chunk_count()).await?,
        )
    };
    Ok(Hff::new(TokioReader::new(source), header, tables, chunks))
}

/// Reads an entire Hff into memory.
pub async fn read(
    mut source: &mut (dyn AsyncRead + std::marker::Unpin),
) -> Result<Hff<ChunkCache>> {
    let header = TokioReader::read_header(&mut source).await?;
    let (tables, chunks) = if header.is_native_endian() {
        (
            TokioReader::read_tables::<NE>(&mut source, header.table_count()).await?,
            TokioReader::read_chunks::<NE>(&mut source, header.chunk_count()).await?,
        )
    } else {
        (
            TokioReader::read_tables::<OP>(&mut source, header.table_count()).await?,
            TokioReader::read_chunks::<OP>(&mut source, header.chunk_count()).await?,
        )
    };
    let cache = TokioReader::read_body(&mut source, tables.len(), chunks.len()).await?;
    Ok(Hff::new(cache, header, tables, chunks))
}

/// Read the structure of a Hff into memory.  Provides access
/// only to the structure without any of the metadata or chunk
/// data available.
pub async fn inspect(
    mut source: &mut (dyn AsyncRead + std::marker::Unpin),
) -> Result<Hff<Inspection>> {
    let header = TokioReader::read_header(&mut source).await?;
    let (tables, chunks) = if header.is_native_endian() {
        (
            TokioReader::read_tables::<NE>(&mut source, header.table_count()).await?,
            TokioReader::read_chunks::<NE>(&mut source, header.chunk_count()).await?,
        )
    } else {
        (
            TokioReader::read_tables::<OP>(&mut source, header.table_count()).await?,
            TokioReader::read_chunks::<OP>(&mut source, header.chunk_count()).await?,
        )
    };
    Ok(Hff::new(Inspection, header, tables, chunks))
}
