use crate::{ReadSeek, StdReader};
use hff_core::{
    byteorder::ReadBytesExt,
    read::{Hff, Inspection},
    ByteOrder, Chunk, ChunkCache, Ecc, Endian, Error, Header, Result, Table, Version, BE, LE, NE,
    OP,
};
use std::{io::Read, mem::size_of};

/// Opens the input and maintains it for random access to the
/// metadata and chunks.
pub fn open(mut source: impl ReadSeek + 'static) -> Result<Hff<StdReader>> {
    let (header, tables, chunks) = read_hff(&mut source)?;
    Ok(Hff::new(StdReader::new(source), header, tables, chunks))
}

/// Reads an entire Hff into memory.
pub fn read(source: &mut dyn Read) -> Result<Hff<ChunkCache>> {
    let (header, tables, chunks, cache) = read_hff_full(source)?;
    Ok(Hff::new(cache, header, tables, chunks))
}

/// Read the structure of a Hff into memory.  Provides access
/// only to the structure without any of the metadata or chunk
/// data available.
pub fn inspect(source: &mut dyn Read) -> Result<Hff<Inspection>> {
    let (header, tables, chunks) = read_hff(source)?;
    Ok(Hff::new(Inspection, header, tables, chunks))
}

// Helpers to read hff from std::io::Read traits.

pub(super) fn read_hff(reader: &mut dyn Read) -> Result<(Header, Vec<Table>, Vec<Chunk>)> {
    // The header determines the structure endianess.
    let header = read_header(reader)?;
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

    Ok((header, tables, chunks))
}

fn read_hff_full(reader: &mut dyn Read) -> Result<(Header, Vec<Table>, Vec<Chunk>, ChunkCache)> {
    let (header, tables, chunks) = read_hff(reader)?;

    let mut buffer = vec![];
    reader.read_to_end(&mut buffer)?;

    let offset = size_of::<Header>()
        + (size_of::<Table>() * tables.len())
        + (size_of::<Chunk>() * chunks.len());
    let cache = ChunkCache::new(offset, buffer);

    Ok((header, tables, chunks, cache))
}

/// Read the header from a given stream.
fn read_header(reader: &mut dyn Read) -> Result<Header> {
    let mut magic = [0_u8; 8];
    reader.read_exact(&mut magic)?;

    // Detect the file content endianess.  NOTE: This only describes
    // the file structure itself, the chunk content is "not" considered
    // part of this.  It is up to the user to deal with endianess of
    // the chunks.
    match Ecc::HFF_MAGIC.endian(magic.into()) {
        Some(endian) => match endian {
            Endian::Little => Ok(Header::with(
                magic.into(),
                Version::read::<LE>(reader)?,
                reader.read_u32::<LE>()?,
                Ecc::read::<LE>(reader)?,
                reader.read_u32::<LE>()?,
                reader.read_u32::<LE>()?,
            )),
            Endian::Big => Ok(Header::with(
                magic.into(),
                Version::read::<BE>(reader)?,
                reader.read_u32::<BE>()?,
                Ecc::read::<BE>(reader)?,
                reader.read_u32::<BE>()?,
                reader.read_u32::<BE>()?,
            )),
        },
        None => Err(Error::Invalid("Not an HFF file.".into())),
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
