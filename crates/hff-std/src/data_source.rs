use super::{Error, Result};
use std::{
    fmt::Debug,
    fs::File,
    path::{Path, PathBuf},
};

/// The source of data for given metadata or chunk.
#[derive(Debug)]
pub enum DataSource<'a> {
    /// Data owned in the data source.
    Owned(Vec<u8>),
    /// Data referred to by the data source.
    Ref(&'a [u8]),
    /// An open file and the length of the data contained within it.
    File(File, u64),
    /// A compressed chunk data source.
    #[cfg(feature = "compression")]
    Compressed(u32, Option<Box<DataSource<'a>>>, Option<Vec<u8>>),
}

impl<'a> DataSource<'a> {
    /// Create a new owned data source.
    pub fn owned(data: impl Into<Vec<u8>>) -> Self {
        Self::Owned(data.into())
    }

    /// Create a data source referencing a u8 array.
    pub fn reference(data: &'a [u8]) -> Self {
        Self::Ref(data)
    }

    /// Create a new file data source.
    pub fn file(source: File, len: u64) -> Self {
        Self::File(source, len)
    }

    /// Create a new compressed data source.
    #[cfg(feature = "compression")]
    pub fn compressed(level: u32, source: DataSource<'a>) -> Self {
        Self::Compressed(level, Some(Box::new(source)), None)
    }

    /// Get the length of the content if known at this time.
    pub fn len(&self) -> Option<usize> {
        match self {
            Self::Owned(d) => Some(d.len()),
            Self::Ref(d) => Some(d.len()),
            Self::File(_, l) => Some(*l as usize),
            #[cfg(feature = "compression")]
            Self::Compressed(_, _, data) => {
                if let Some(data) = data {
                    Some(data.len())
                } else {
                    None
                }
            }
        }
    }

    /// Prepare the content of the data.
    /// For compressed content this runs the default compression
    /// and turns this into an Owned variant.
    pub fn prepare(&mut self) -> Result<u64> {
        match self {
            #[cfg(feature = "compression")]
            Self::Compressed(level, source, data) => {
                // Take the source item and collapse it into the owned data entry
                // if needed.  (Prepare is re-entrant and could be called several
                // times.)
                if let Some(source) = source.take() {
                    let source = *source;
                    match source {
                        DataSource::Owned(d) => *data = Some(d),
                        DataSource::Ref(d) => *data = Some(d.into()),
                        DataSource::File(mut f, _) => {
                            let mut buffer = vec![];
                            std::io::copy(&mut f, &mut buffer)?;
                            *data = Some(buffer);
                        }
                        _ => panic!(
                            "Invalid data source.  Compressing compressed data for instance."
                        ),
                    }
                } else {
                    // Already prepared or internal error.
                    if let Some(data) = data {
                        return Ok(data.len() as u64);
                    } else {
                        return Err(Error::Invalid(
                            "Internal error dealing with compressed data.".into(),
                        ));
                    }
                };

                // Compress it and replace data.
                let mut encoder = xz2::write::XzEncoder::new(vec![], (*level).min(9));
                let source = data.take().unwrap();
                std::io::copy(&mut source.as_slice(), &mut encoder)?;
                *data = Some(encoder.finish()?);

                Ok(data.as_ref().unwrap().len() as u64)
            }
            // Everything else can be used as is.
            _ => Ok(self.len().unwrap() as u64),
        }
    }
}

impl<'a> TryInto<DataSource<'a>> for &str {
    type Error = Error;

    fn try_into(self) -> std::prelude::v1::Result<DataSource<'a>, Self::Error> {
        Ok(DataSource::Owned(self.as_bytes().into()))
    }
}

impl<'a> TryInto<DataSource<'a>> for &[u8] {
    type Error = Error;

    fn try_into(self) -> std::prelude::v1::Result<DataSource<'a>, Self::Error> {
        Ok(DataSource::Owned(self.into()))
    }
}

impl<'a> TryInto<DataSource<'a>> for Vec<u8> {
    type Error = Error;

    fn try_into(self) -> std::prelude::v1::Result<DataSource<'a>, Self::Error> {
        Ok(DataSource::Owned(self))
    }
}

impl<'a> TryInto<DataSource<'a>> for &Path {
    type Error = Error;

    fn try_into(self) -> std::prelude::v1::Result<DataSource<'a>, Self::Error> {
        let file = std::fs::File::open(self)?;
        let length = file.metadata()?.len();
        Ok(DataSource::File(file, length))
    }
}

impl<'a> TryInto<DataSource<'a>> for PathBuf {
    type Error = Error;

    fn try_into(self) -> std::prelude::v1::Result<DataSource<'a>, Self::Error> {
        let file = std::fs::File::open(&self)?;
        let length = file.metadata()?.len();
        Ok(DataSource::File(file, length))
    }
}

#[cfg(feature = "compression")]
impl<'a, T> TryInto<DataSource<'a>> for (u32, T)
where
    T: TryInto<DataSource<'a>>,
    <T as TryInto<DataSource<'a>>>::Error: std::fmt::Debug,
    Error: From<<T as TryInto<DataSource<'a>>>::Error>,
{
    type Error = Error;

    fn try_into(self) -> std::prelude::v1::Result<DataSource<'a>, Self::Error> {
        Ok(DataSource::Compressed(
            self.0,
            Some(Box::new(self.1.try_into()?)),
            None,
        ))
    }
}
