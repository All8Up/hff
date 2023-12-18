use super::TableDesc;
use hff_core::{ByteOrder, Ecc, Header, Result, Table};

/// Write the table structure to an HFF container.
pub fn write_stream<E: ByteOrder>(
    content_type: impl Into<Ecc>,
    content: TableDesc,
    writer: &mut dyn std::io::Write,
) -> Result<()> {
    let table_count = content.table_count();
    let chunk_count = content.chunk_count();
    let header = Header::new(content_type.into(), table_count as u32, chunk_count as u32);

    // Write the header.
    header.write::<E>(writer)?;
    // Write the tables.
    // NOTE: When chunks are compressed this will force them into memory
    // because we did not specify the writer to be seekable.
    // TODO: Add a seekable variation so we can update the table chunk
    // information after writing the chunks.
    let (parent, children) = content.flatten_tables();
    write_tables::<E>(parent, children, writer)?;
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
