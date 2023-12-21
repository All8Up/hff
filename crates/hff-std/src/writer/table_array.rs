use super::TableDesc;
use crate::{DataSource, Error, Result};
use hff_core::{ByteOrder, Chunk, Ecc, Table};
use std::{
    io::Write,
    ops::{Index, IndexMut},
};

/// The table array to be written.
#[derive(Debug)]
pub struct TableArray {
    /// A flag indicating if the table had attached metadata,
    /// and the table structure itself.
    tables: Vec<(bool, Table)>,
}

impl TableArray {
    /// Create a new empty table array.
    pub fn new() -> Self {
        Self { tables: vec![] }
    }

    /// Get the number of tables in the array.
    pub fn len(&self) -> usize {
        self.tables.len()
    }

    /// Push a new table into the array.
    pub fn push(&mut self, has_metadata: bool, table: Table) {
        self.tables.push((has_metadata, table));
    }

    /// Get the last table in the vector mutably.
    pub fn last_mut(&mut self) -> Option<&mut (bool, Table)> {
        self.tables.last_mut()
    }

    /// Write the tables to the given stream.
    pub fn write<E: ByteOrder>(self, writer: &mut dyn Write) -> Result<()> {
        for table in self.tables {
            table.1.write::<E>(writer)?;
        }
        Ok(())
    }
}

impl Index<usize> for TableArray {
    type Output = (bool, Table);

    fn index(&self, index: usize) -> &Self::Output {
        &self.tables[index]
    }
}

impl IndexMut<usize> for TableArray {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.tables[index]
    }
}
