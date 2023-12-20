use hff_core::{ByteOrder, Chunk, Ecc, Header, Result, Table, NE, OP};
use std::{
    fmt::Debug,
    io::{Cursor, Read, Seek, SeekFrom},
    mem::size_of,
};

/// Trait for types that can be used as a source of chunks.
pub trait ReadSeek: Read + Seek {}

// Implement over anything which can read and seek.
impl<T: Read + Seek> ReadSeek for T {}

/// Act as a read/seek IO object for purposes of having
/// an entire HFF in memory at one time.
#[derive(Debug, Clone)]
pub struct ChunkCache {
    offset: u64,
    buffer: Cursor<Vec<u8>>,
}

impl ChunkCache {
    /// Create a new chunk cache.
    pub fn new(offset: usize, buffer: Vec<u8>) -> Self {
        Self {
            offset: offset as u64,
            buffer: Cursor::new(buffer),
        }
    }
}

impl Read for ChunkCache {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.buffer.read(buf)
    }
}

impl Seek for ChunkCache {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        // Adjust the position if it is from the start because we want
        // to act as if we are in the file 'after' the header+tables.
        let pos = match pos {
            SeekFrom::Current(p) => SeekFrom::Current(p),
            SeekFrom::Start(p) => SeekFrom::Start(p - self.offset),
            SeekFrom::End(p) => SeekFrom::End(p),
        };
        self.buffer.seek(pos)
    }
}

///
#[derive(Debug, Clone, PartialEq)]
pub struct Hff {
    tables: Vec<Table>,
    chunks: Vec<Chunk>,
}

impl Hff {
    /// Create a new Hff wrapper.
    pub fn new(tables: impl Into<Vec<Table>>, chunks: impl Into<Vec<Chunk>>) -> Self {
        Self {
            tables: tables.into(),
            chunks: chunks.into(),
        }
    }

    /// Get the offset from the start of the file to the start of the chunk data.
    pub fn offset_to_data(&self) -> usize {
        size_of::<Header>()
            + (size_of::<Table>() * self.tables.len())
            + (size_of::<Chunk>() * self.chunks.len())
    }

    /// Get an iterator over the root level of the content.
    pub fn iter(&self) -> TableIter {
        TableIter::new(self, 0)
    }
}

///
pub struct TableView<'a> {
    hff: &'a Hff,
    index: usize,
}

impl<'a> PartialEq for TableView<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.hff == other.hff && self.index == other.index
    }
}

impl<'a> Debug for TableView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.hff.tables[self.index])
    }
}

impl<'a> TableView<'a> {
    /// Create a new TableView.
    pub(super) fn new(hff: &'a Hff, index: usize) -> Self {
        Self { hff, index }
    }

    /// Get the primary identifier.
    pub fn primary(&self) -> Ecc {
        self.hff.tables[self.index].primary()
    }

    /// Get the secondary identifier.
    pub fn secondary(&self) -> Ecc {
        self.hff.tables[self.index].secondary()
    }

    /// Get the count of child tables.
    pub fn child_count(&self) -> usize {
        self.hff.tables[self.index].child_count() as usize
    }

    /// Get an iterator to the child tables.
    pub fn iter(&self) -> TableIter<'a> {
        TableIter::new(self.hff, self.index + 1)
    }

    /// Get an iterator of the chunks.
    pub fn chunks(&self) -> ChunkIter<'a> {
        let table = &self.hff.tables[self.index];
        ChunkIter::new(
            self.hff,
            table.chunk_index() as usize,
            table.chunk_count() as usize,
        )
    }

    /// Get the count of chunks in the table.
    pub fn chunk_count(&self) -> usize {
        self.hff.tables[self.index].chunk_count() as usize
    }

    /// Read the metadata from the given source.
    pub fn metadata(&self, source: &mut dyn ReadSeek) -> Result<Vec<u8>> {
        let table = &self.hff.tables[self.index];
        if table.metadata_length() > 0 {
            source.seek(SeekFrom::Start(table.metadata_offset()))?;
            let mut buffer = vec![0; table.metadata_length() as usize];
            source.read_exact(buffer.as_mut_slice())?;
            Ok(buffer)
        } else {
            Ok(vec![])
        }
    }
}

pub struct ChunkView<'a> {
    hff: &'a Hff,
    index: usize,
}

impl<'a> ChunkView<'a> {
    /// Create a new view.
    pub fn new(hff: &'a Hff, index: usize) -> Self {
        Self { hff, index }
    }

    /// Get the primary identifier.
    pub fn primary(&self) -> Ecc {
        self.hff.chunks[self.index].primary()
    }

    /// Get the secondary identifier.
    pub fn secondary(&self) -> Ecc {
        self.hff.chunks[self.index].secondary()
    }

    /// Get the size of the data in the chunk.
    pub fn size(&self) -> usize {
        self.hff.chunks[self.index].length() as usize
    }

    /// Get the data for the chunk from a read/seek source.
    pub fn data(&self, source: &mut dyn ReadSeek) -> Result<Vec<u8>> {
        let chunk = &self.hff.chunks[self.index];
        if chunk.length() > 0 {
            source.seek(SeekFrom::Start(chunk.offset()))?;
            let mut buffer = vec![0; chunk.length() as usize];
            source.read_exact(buffer.as_mut_slice())?;
            Ok(buffer)
        } else {
            Ok(vec![])
        }
    }
}

///
pub struct ChunkIter<'a> {
    hff: &'a Hff,
    current: isize,
    count: usize,
}

impl<'a> ChunkIter<'a> {
    /// Create a new chunk iterator.
    pub fn new(hff: &'a Hff, index: usize, count: usize) -> Self {
        Self {
            hff,
            current: index as isize - 1,
            count,
        }
    }
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = ChunkView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            self.count -= 1;
            self.current += 1;
            Some(ChunkView::new(self.hff, self.current as usize))
        } else {
            None
        }
    }
}

///
pub struct TableIter<'a> {
    hff: &'a Hff,
    start: Option<usize>,
    index: usize,
}

impl<'a> TableIter<'a> {
    /// Create a new table iterator.
    pub fn new(hff: &'a Hff, start: usize) -> Self {
        Self {
            hff,
            start: Some(start),
            index: 0,
        }
    }
}

impl<'a> Iterator for TableIter<'a> {
    type Item = TableView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(start) = self.start.take() {
            self.index = start;
            Some(TableView::new(self.hff, self.index))
        } else {
            let sibling = self.hff.tables[self.index].sibling() as usize;
            if sibling == 0 {
                None
            } else {
                let index = self.index + sibling;
                if index >= self.hff.tables.len() {
                    None
                } else {
                    self.index = index;
                    Some(TableView::new(self.hff, self.index))
                }
            }
        }
    }
}

/// Read a HFF from the given stream.
pub fn read_stream(reader: &mut dyn Read) -> Result<Hff> {
    // The header determines the structure endianess.
    let header = Header::read(reader)?;
    if header.is_native_endian() {
        println!("Native endian.");
        let tables = read_tables::<NE>(reader, header.table_count())?;
        let chunks = read_chunks::<NE>(reader, header.chunk_count())?;
        Ok(Hff::new(tables, chunks))
    } else {
        println!("Opposing endian.");
        let tables = read_tables::<OP>(reader, header.table_count())?;
        let chunks = read_chunks::<OP>(reader, header.chunk_count())?;
        Ok(Hff::new(tables, chunks))
    }
}

/// Read a HFF from the given stream along with all the data for the chunks.
pub fn read_stream_full(reader: &mut dyn Read) -> Result<(Hff, ChunkCache)> {
    let hff = read_stream(reader)?;
    let mut buffer = vec![];
    reader.read_to_end(&mut buffer)?;

    let offset = hff.offset_to_data();
    Ok((hff, ChunkCache::new(offset, buffer)))
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
