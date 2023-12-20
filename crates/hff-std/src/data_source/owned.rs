use super::{DataSource, StdWriter};
use crate::{Error, Result};
use std::{fmt::Debug, io::Write};

// Internal structure to deal with raw data sources such as
// vectors, strings etc that are directly available in memory.
pub(super) struct OwnedDataSource(Vec<u8>);

impl Debug for OwnedDataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<OwnedDataSource>")
    }
}

impl DataSource for OwnedDataSource {
    fn len(&self) -> Option<u64> {
        Some(self.0.len() as u64)
    }

    fn prepare(&mut self) -> Result<u64> {
        Ok(self.0.len() as u64)
    }

    fn as_datatype(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl StdWriter for OwnedDataSource {
    fn write(&mut self, writer: &mut dyn Write) -> Result<()> {
        std::io::copy(&mut self.0.as_slice(), writer)?;
        Ok(())
    }
}

impl TryInto<Box<dyn DataSource>> for &str {
    type Error = Error;

    fn try_into(self) -> Result<Box<dyn DataSource>> {
        Ok(Box::new(OwnedDataSource(self.as_bytes().into())))
    }
}

impl TryInto<Box<dyn DataSource>> for &[u8] {
    type Error = Error;

    fn try_into(self) -> Result<Box<dyn DataSource>> {
        Ok(Box::new(OwnedDataSource(self.into())))
    }
}

impl TryInto<Box<dyn DataSource>> for Vec<u8> {
    type Error = Error;

    fn try_into(self) -> Result<Box<dyn DataSource>> {
        Ok(Box::new(OwnedDataSource(self)))
    }
}

// An owned data source will immediately consume the entire
// content of the read stream and store it in memory.  Avoid
// this unless you don't care about memory or you know the
// stream is small.  Obviously the Read needs to complete or
// this will block.
use std::io::Read;

impl TryInto<Box<dyn DataSource>> for &mut dyn Read {
    type Error = Error;

    fn try_into(self) -> Result<Box<dyn DataSource>> {
        let mut buffer = vec![];
        self.read_to_end(&mut buffer)?;

        Ok(Box::new(OwnedDataSource(buffer)))
    }
}
