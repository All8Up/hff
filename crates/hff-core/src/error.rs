use thiserror::Error;

/// Common error type.
#[derive(Debug, Error)]
pub enum Error {
    /// Infallible errors.  I.e. they better never be hit.
    #[error("{0}")]
    Infallible(#[from] std::convert::Infallible),
    /// Invalid content.
    #[error("{0}")]
    Invalid(String),
    /// Item not found.
    #[error("{0}")]
    NotFound(String),
    /// Utf8 parsing error.
    #[error("{0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    /// The given string to the Ecc is invalid.
    #[error("{0}")]
    InvalidEcc(String),
    /// A child table has an invalid data source.
    #[error("{0}")]
    InvalidTableData(String),
    /// Structural error when building an rcff container.
    /// Metadata is only allowed once on each table within the container.
    #[error("{0}")]
    DuplicateMetadata(String),
    /// File manipulation error.
    #[error("{0}")]
    StripPrefixError(#[from] std::path::StripPrefixError),
    /// An IO error.
    #[error("{0}")]
    IoError(#[from] std::io::Error),
}

/// The standard result type used in the crate.
pub type Result<T> = std::result::Result<T, crate::Error>;
