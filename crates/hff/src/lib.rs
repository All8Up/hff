//! The high level wrapper around the supporting HFF crates.
//!
//! # Examples
//!
//! ```
//! use hff_std::*;
//!
//! // Creating the content can use the builder:
//! let content = hff([
//!     table((Ecc::new("Prime"), Ecc::new("Second")))
//!     // Metadata and chunks can be pulled from many types of source data.
//!     .metadata("Each table can have metadata.").unwrap()
//!     // Tables can have chunks.
//!     .chunks([
//!         chunk((Ecc::new("AChunk"), Ecc::INVALID), "Each table can have 0..n chunks of data.").unwrap()
//!     ])
//!     // Tables can have child tables.
//!     .children([
//!         table((Ecc::new("Child1"), Ecc::INVALID))
//!         .metadata("Unique to this table.").unwrap()
//!         .chunks([
//!             chunk((Ecc::new("ThisFile"), Ecc::new("Copy")), "more data").unwrap()
//!         ])
//!     ]),
//!     // And there can be multiple tables at the root.
//!     table((Ecc::new("Child2"), Ecc::INVALID))
//! ]);
//!
//! // The results can be packaged into an output stream.
//! // This can be anything which supports the std::io::Write trait.
//! let mut buffer = vec![];
//! content.write::<NE>(IdType::Ecc2, "Test", &mut buffer).unwrap();
//!
//! // Hff can be read back from anything which supports the std::io::Read
//! // trait.  In this case we also read all the data into a cache in memory.
//! // The cache is simply an array with Read+Seek implemented on top of a
//! // Vec<u8>.
//! let hff = read(&mut buffer.as_slice()).unwrap();
//!
//! // The Hff instance contains the structure of the content and can be
//! // iterated in multiple ways.  Here, we'll use the depth first iterator
//! // just to see all the content.
//! for (depth, table) in hff.depth_first() {
//!     // Print information about the table.
//!     let metadata = hff.read(&table).unwrap_or(&[0; 0]);
//!     println!("{}: {:?} ({})",
//!         depth,
//!         table.identifier(),
//!         std::str::from_utf8(metadata).unwrap()
//!     );
//!
//!     // Iterate the chunks.
//!     for chunk in table.chunks() {
//!         println!("{}", std::str::from_utf8(hff.read(&chunk).unwrap()).unwrap());
//!     }
//! }
//! ```
//!
//! # In Progress
//! - [x] Depth first iterator through tables.
//! - [x] More metadata/chunk data source types.  Most things which can be turned into
//! Vec<u8> exist now, read trait for anything which can be immediately pulled in at
//! runtime and finally std::path::{Path, PathBuf} to pull data from a file.
//! - [x] Yet more metadata/chunk data source types.
//! Compression is done and uses lzma due to the desired performance versus compression.
//! Pass in a tuple with: (level, any valid data source) where level is 0-9.
//! - [x] Utility types for metadata.  For instance a simple key=value string map and a
//! simple array of strings.
//! - [x] Change the table builder to allow multiple tables at the 'root' level.
//! Currently the builder expects a single outer table to contain all others.  This
//! is a holdover from a prior format structure which was removed.
//! - [x] After fixing the table builder, implement the lazy header variation so compressed
//! chunks do not have to be stored in memory prior to writing.
//! - [ ] Remove the development testing and write better and more complete tests.
//! - [ ] Better examples.
//! - [x] Async-std implementation of the reader.
//! - [ ] Async-std implementation of the writer.
//! - [x] Tokio implementation of the reader.
//! - [ ] Tokio implementation of the writer.
//! - [ ] Mmap, io_ring and whatever other variations make sense in the long run.
#![warn(missing_docs)]

// Core types.
#[doc(inline)]
pub use hff_core::*;

#[doc(inline)]
pub use hff_std;

#[cfg(feature = "async-std-rt")]
#[doc(inline)]
pub use hff_async_std;

#[cfg(feature = "tokio-rt")]
#[doc(inline)]
pub use hff_tokio;
