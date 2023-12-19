use crate::{Ecc, Result};
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::{
    fmt::Debug,
    io::{Read, Write},
};

/// Specifies a chunk of data within the file.
#[repr(C, align(16))]
#[derive(Copy, Clone, PartialEq, Hash)]
pub struct Chunk {
    /// Primary content type of the chunk data.
    primary: Ecc,
    /// Secondary content type of the chunk data.
    secondary: Ecc,
    /// Length of the chunk data.  (Not rounded to 16 bytes.)
    length: u64,
    /// Offset of the data from the start of the file.
    offset: u64,
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}:{}) - {}, {}",
            self.primary.to_string(),
            self.secondary.to_string(),
            self.length,
            self.offset
        )
    }
}

impl Chunk {
    /// Create a new chunk instance.
    pub fn new(
        primary: impl Into<Ecc>,
        secondary: impl Into<Ecc>,
        length: u64,
        offset: u64,
    ) -> Self {
        Self {
            primary: primary.into(),
            secondary: secondary.into(),
            length,
            offset,
        }
    }

    /// Get the primary content type.
    pub fn primary(&self) -> Ecc {
        self.primary
    }

    /// Get the secondary content type.
    pub fn secondary(&self) -> Ecc {
        self.secondary
    }

    /// Get the length of the content.
    pub fn length(&self) -> u64 {
        self.length
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
            primary: Ecc::read::<E>(reader)?,
            secondary: Ecc::read::<E>(reader)?,
            length: reader.read_u64::<E>()?,
            offset: reader.read_u64::<E>()?,
        })
    }

    /// Write a table to the given stream.
    pub fn write<E: ByteOrder>(self, writer: &mut dyn Write) -> Result<()> {
        self.primary.write::<E>(writer)?;
        self.secondary.write::<E>(writer)?;
        writer.write_u64::<E>(self.length)?;
        writer.write_u64::<E>(self.offset)?;

        Ok(())
    }
}
