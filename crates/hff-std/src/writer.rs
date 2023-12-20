use crate::{DataBuilder, TableDesc};
use hff_core::{ByteOrder, Chunk, Ecc, Header, Result, Table};

/// Write the table structure to an HFF container.
/// NOTE: This does not support seek streams and as
/// such any compressed chunks will need to be buffered
/// in memory until written.  Seek will allow deferral
/// of the compression and reduce memory requirements.
/// Support for seek and lazy header updates will be
/// added later.
/// NOTE: Currently this expects a single root table to
/// encapsulate everything in the file.  This is not a
/// rule in the file format, it is simply how this is
/// written.
pub fn write_stream<E: ByteOrder>(
    content_type: impl Into<Ecc>,
    content: TableDesc,
    writer: &mut dyn std::io::Write,
) -> Result<()> {
    // Compute the header information.
    let table_count = content.table_count();
    let chunk_count = content.chunk_count();
    let header = Header::new(content_type.into(), table_count as u32, chunk_count as u32);

    // Write the header.
    header.write::<E>(writer)?;
    // Write the tables.
    let (tables, chunks, data) = content.flatten_tables()?.finish();

    write_tables::<E>(tables, writer)?;
    write_chunks::<E>(chunks, writer)?;
    write_data(data, writer)?;

    writer.flush()?;
    Ok(())
}

fn write_tables<E: ByteOrder>(tables: Vec<Table>, writer: &mut dyn std::io::Write) -> Result<()> {
    for table in tables {
        table.write::<E>(writer)?;
    }
    Ok(())
}

fn write_chunks<E: ByteOrder>(chunks: Vec<Chunk>, writer: &mut dyn std::io::Write) -> Result<()> {
    for chunk in chunks {
        chunk.write::<E>(writer)?;
    }
    Ok(())
}

fn write_data(data: DataBuilder, writer: &mut dyn std::io::Write) -> Result<()> {
    data.write(writer)
}
