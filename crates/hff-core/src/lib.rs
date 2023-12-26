//! HFF Core
//! Contains the internal structure of HFF and basic
//! serialization abilities.
#![warn(missing_docs)]

// Endian utilities.
mod endian;
pub use endian::*;

// The crate error and result types.
mod error;
pub use error::{Error, Result};

// The eight character code type.
mod ecc;
pub use ecc::Ecc;

// The semantic versioning type.
mod semver;
pub use semver::Semver;

// The file header.
mod header;
pub use header::Header;

// A table in the structure.
mod table;
pub use table::Table;

// A chunk in the structure.
mod chunk;
pub use chunk::Chunk;

// The read container.
mod hff;
pub use hff::{ChunkIter, ChunkView, DepthFirstIter, Hff, TableIter, TableView};

// Re-export byte order.
pub use byteorder;
