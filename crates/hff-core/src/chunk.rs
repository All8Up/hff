use crate::{Error, Identifier, Result};
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::{
    fmt::Debug,
    io::{Read, Write},
};

/// Specifies a chunk of data within the file.
#[repr(C, align(16))]
#[derive(Copy, Clone, PartialEq, Hash)]
pub struct Chunk {
    /// The identifier for the chunk.
    identifier: Identifier,
    /// Length of the chunk data.  (Not rounded to 16 bytes.)
    length: u64,
    /// Offset of the data from the start of the file.
    offset: u64,
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({:X}) - {}, {}",
            *self.identifier, self.length, self.offset
        )
    }
}

impl Chunk {
    /// Size of the chunk entry.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Create a new chunk instance.
    pub fn new(
        identifier: impl TryInto<Identifier, Error = Error>,
        length: u64,
        offset: u64,
    ) -> Self {
        Self {
            identifier: identifier
                .try_into()
                .expect("Expected valid identifier data."),
            length,
            offset,
        }
    }

    /// Get the identifier.
    pub fn identifier(&self) -> Identifier {
        self.identifier
    }

    /// Get the length of the content.
    pub fn length(&self) -> u64 {
        self.length
    }

    /// Get the length mutably.
    pub fn length_mut(&mut self) -> &mut u64 {
        &mut self.length
    }

    /// Get the offset of the content.
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Get the offset mutably.
    pub fn offset_mut(&mut self) -> &mut u64 {
        &mut self.offset
    }

    /// Read a table from the given stream.
    pub fn read<E: ByteOrder>(reader: &mut dyn Read) -> Result<Self> {
        Ok(Self {
            identifier: reader.read_u128::<E>()?.try_into()?,
            length: reader.read_u64::<E>()?,
            offset: reader.read_u64::<E>()?,
        })
    }

    /// Write a table to the given stream.
    pub fn write<E: ByteOrder>(self, writer: &mut dyn Write) -> Result<()> {
        writer.write_u128::<E>(*self.identifier)?;
        writer.write_u64::<E>(self.length)?;
        writer.write_u64::<E>(self.offset)?;

        Ok(())
    }
}
