use hff_core::{ByteOrder, Header, Result, Table, NE};
use std::mem::size_of;

/// Read a HFF from the given stream.
pub fn read_stream(reader: &mut dyn std::io::Read) -> Result<()> {
    // The header determines the structure endianess.
    let header = Header::read(reader)?;
    if header.is_native_endian() {
        //
        println!("Native endian.");
        let _tables = read_tables::<NE>(reader, header.table_count())?;
    } else {
        //
        println!("Opposing endian.");
    }
    Ok(())
}

fn read_tables<E: ByteOrder>(reader: &mut dyn std::io::Read, count: u32) -> Result<Vec<Table>> {
    if count > 0 {
        // Create a buffer with the array size in bytes and read it.
        let mut buffer = vec![0; count as usize * size_of::<Table>()];
        reader.read_exact(&mut buffer.as_mut_slice())?;

        // Read all the tables out of the buffer.
        let mut tables = vec![];
        for _ in 0..count {
            let table = Table::read::<E>(&mut buffer.as_slice())?;
            println!("{:#?}", table);
            tables.push(table);
        }

        Ok(tables)
    } else {
        // TODO: Does an empty file make sense?  It's not an error but ....
        Ok(vec![])
    }
}
