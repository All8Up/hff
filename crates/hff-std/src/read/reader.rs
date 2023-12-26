use super::ChunkCache;
use hff_core::{
    byteorder::ReadBytesExt, read::Hff, ByteOrder, Chunk, Ecc, Endian, Error, Header, Result,
    Semver, Table, BE, LE, NE, OP,
};
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
                Semver::read::<LE>(reader)?,
                Ecc::read::<LE>(reader)?,
                reader.read_u32::<LE>()?,
                reader.read_u32::<LE>()?,
            )),
            Endian::Big => Ok(Header::with(
                magic.into(),
                Semver::read::<BE>(reader)?,
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
