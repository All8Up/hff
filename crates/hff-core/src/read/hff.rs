use super::{DepthFirstIter, TableIter};
use crate::{Chunk, Ecc, Header, Table, Version};
use std::{
    fmt::Debug,
    mem::size_of,
    ops::{Deref, DerefMut},
};

/// The Hff structure data.  This is an immutable representation of the
/// content of an Hff stream.
#[derive(Debug)]
pub struct Hff<T: Debug> {
    /// Was the structure in native endian?
    native: bool,
    /// The version of the file format.
    version: Version,
    /// Content type of the hff.
    content_type: Ecc,
    /// The tables found in the header structure.
    tables: Vec<Table>,
    /// The chunks found within the header structure.
    chunks: Vec<Chunk>,
    /// Access system for the underlying stream.
    accessor: T,
}

impl<T: Debug> Hff<T> {
    /// Create a new Hff wrapper.
    pub fn new(
        accessor: T,
        header: Header,
        tables: impl Into<Vec<Table>>,
        chunks: impl Into<Vec<Chunk>>,
    ) -> Self {
        Self {
            native: header.is_native_endian(),
            version: header.version(),
            content_type: header.content_type(),
            tables: tables.into(),
            chunks: chunks.into(),
            accessor,
        }
    }

    /// Return if the structure of the source was in native endian.
    pub fn is_native_endian(&self) -> bool {
        self.native
    }

    /// Return the version of the file structure the file was read from.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Get the content type of the container.
    pub fn content_type(&self) -> Ecc {
        self.content_type
    }

    /// Get the offset from the start of the file to the start of the chunk data.
    pub fn offset_to_data(&self) -> usize {
        size_of::<Header>()
            + (size_of::<Table>() * self.tables.len())
            + (size_of::<Chunk>() * self.chunks.len())
    }

    /// Get an iterator over the tables in depth first order.
    pub fn depth_first(&self) -> DepthFirstIter<'_, T> {
        DepthFirstIter::new(self)
    }

    /// Get an iterator over the child tables.
    pub fn tables(&self) -> TableIter<'_, T> {
        TableIter::new(self, 0)
    }

    /// Get access to the table array.
    pub fn tables_array(&self) -> &[Table] {
        &self.tables
    }

    /// Get access to the chunk array.
    pub fn chunks_array(&self) -> &[Chunk] {
        &self.chunks
    }
}

impl<T: Debug> Deref for Hff<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.accessor
    }
}

impl<T: Debug> DerefMut for Hff<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.accessor
    }
}
