use hff_core::{ByteOrder, Chunk, Ecc, Header, Result, Table, NE, OP};
use std::{fmt::Debug, io::Read, mem::size_of};

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

    /// Get the count of chunks in the table.
    pub fn chunk_count(&self) -> usize {
        self.hff.tables[self.index].chunk_count() as usize
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
