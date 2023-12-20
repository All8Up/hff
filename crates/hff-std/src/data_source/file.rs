use super::{DataSource, StdWriter};
use crate::{Error, Result};
use std::{
    fmt::Debug,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

// Internal structure to deal with raw data sources such as
// vectors, strings etc that are directly available in memory.
pub(super) struct FileDataSource(File, u64);

impl Debug for FileDataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<FileDataSource>")
    }
}

impl DataSource for FileDataSource {
    fn len(&self) -> Option<u64> {
        Some(self.1 as u64)
    }

    fn prepare(&mut self) -> Result<u64> {
        Ok(self.1 as u64)
    }

    fn as_datatype(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl StdWriter for FileDataSource {
    fn write(&mut self, writer: &mut dyn Write) -> Result<()> {
        std::io::copy(&mut self.0, writer)?;
        Ok(())
    }
}

// NOTE: These leave the file open until they are written out.
// This prevents race conditions but if you are using a lot of
// files, this might run into FD limits on Linux/macOS.

impl TryInto<Box<dyn DataSource>> for &Path {
    type Error = Error;

    fn try_into(self) -> Result<Box<dyn DataSource>> {
        let file = File::open(self)?;
        let size = file.metadata()?.len();

        Ok(Box::new(FileDataSource(file, size)))
    }
}

impl TryInto<Box<dyn DataSource>> for PathBuf {
    type Error = Error;

    fn try_into(self) -> Result<Box<dyn DataSource>> {
        let file = File::open(self)?;
        let size = file.metadata()?.len();

        Ok(Box::new(FileDataSource(file, size)))
    }
}
