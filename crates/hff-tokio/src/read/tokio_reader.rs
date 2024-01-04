use super::ReadSeek;
use hff_core::{
    byteorder::ReadBytesExt, ByteOrder, Chunk, ChunkCache, ContentInfo, Ecc, Endian, Error, Header,
    Result, Semver, Table, BE, LE,
};
use std::mem::size_of;
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt},
    sync::{Mutex, MutexGuard},
};

/// Implements a std reader wrapper around the source.
pub struct TokioReader {
    source: Mutex<Box<dyn ReadSeek>>,
}

impl std::fmt::Debug for TokioReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TokioReader")
    }
}

impl TokioReader {
    /// Create a new std reader type.
    pub fn new(source: impl ReadSeek + 'static) -> Self {
        Self {
            source: Mutex::new(Box::new(source)),
        }
    }

    /// Read the given content.
    pub async fn read(&self, content: &dyn ContentInfo) -> Result<Vec<u8>> {
        let mut source = self.source.lock().await;
        source
            .seek(std::io::SeekFrom::Start(content.offset()))
            .await?;
        let mut result = vec![0; content.len() as usize];
        source.read_exact(result.as_mut_slice()).await?;
        Ok(result)
    }

    /// Get he appropriate reader implementation.
    pub async fn reader(
        &self,
        content: impl ContentInfo,
    ) -> Result<MutexGuard<'_, Box<dyn ReadSeek>>> {
        let mut source = self.source.lock().await;
        source
            .seek(std::io::SeekFrom::Start(content.offset()))
            .await?;
        Ok(source)
    }

    /// Read the header from the given stream.
    pub(super) async fn read_header(
        reader: &mut (dyn tokio::io::AsyncRead + std::marker::Unpin),
    ) -> Result<Header> {
        // Read in the entire header ignorant of the endian.
        let mut header = [0_u8; Header::SIZE];
        reader.read_exact(&mut header).await?;

        // Build a byte reader from the buffer.
        let reader: &mut dyn std::io::Read = &mut header.as_slice();

        // Read out the magic value to detect endian.
        let magic = reader.read_u64::<LE>()?;

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

    /// Read the tables from the given stream.
    pub(super) async fn read_tables<E: ByteOrder>(
        reader: &mut (dyn tokio::io::AsyncRead + std::marker::Unpin),
        count: u32,
    ) -> Result<Vec<Table>> {
        if count > 0 {
            // Create a buffer with appropriate size.
            let mut buffer = vec![0; count as usize * std::mem::size_of::<Table>()];
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

    /// Read the chunks from the given stream.
    pub(super) async fn read_chunks<E: ByteOrder>(
        reader: &mut (dyn tokio::io::AsyncRead + std::marker::Unpin),
        count: u32,
    ) -> Result<Vec<Chunk>> {
        if count > 0 {
            // Create a buffer with the appropriate size.
            let mut buffer = vec![0; count as usize * std::mem::size_of::<Chunk>()];
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

    /// Read the body of data from the given stream.  Assumes the stream
    /// has an 'end' to read to.
    pub(super) async fn read_body(
        reader: &mut (dyn tokio::io::AsyncRead + std::marker::Unpin),
        tables: usize,
        chunks: usize,
    ) -> Result<ChunkCache> {
        let mut buffer = vec![];
        reader.read_to_end(&mut buffer).await?;

        let offset = Header::SIZE + size_of::<Table>() * tables + size_of::<Chunk>() * chunks;
        Ok(ChunkCache::new(offset, buffer))
    }
}
