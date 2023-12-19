use crate::TableDesc;
use hff_core::{ByteOrder, Ecc, Header, Result, Table};

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
    println!("{:#?}", header);

    // Write the header.
    header.write::<E>(writer)?;
    // Write the tables.
    let flattened = content.flatten_tables()?;
    println!("{:#?}", flattened);
    //write_tables::<E>(flattened.root, flattened.children, writer)?;
    // Write the chunks.

    Ok(())
}

fn write_tables<E: ByteOrder>(
    parent: Table,
    children: Vec<Table>,
    writer: &mut dyn std::io::Write,
) -> Result<()> {
    println!("------------ Writing tables.");
    // Write the parent table.
    println!("{}: {:#?}", 0, parent);
    parent.write::<E>(writer)?;

    // Write the children.
    for (index, table) in children.iter().enumerate() {
        println!("{}: {:#?}", index + 1, table);
    }
    for table in children {
        table.write::<E>(writer)?;
    }
    Ok(())
}
