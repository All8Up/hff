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

// We can use a number of different identification
// schemes within tables and chunks.  Different
// identifiers have different purposes.
mod identifier;
pub use identifier::*;

// The eight character code type.
mod ecc;
pub use ecc::Ecc;

// File format versioning.
mod version;
pub use version::Version;

// The file header.
mod header;
pub use header::Header;

// A table in the structure.
mod table;
pub use table::Table;

// A chunk in the structure.
mod chunk;
pub use chunk::Chunk;

// Helper for full file reading.
mod chunk_cache;
pub use chunk_cache::ChunkCache;

// Read support.
pub mod read;

// Write support.
pub mod write;

// Information about metadata or chunk data.
mod content_info;
pub use content_info::ContentInfo;

// General utilities.
pub mod utilities;

// Re-export byte order.
pub use byteorder;

// Re-export uuid.
pub use uuid;
