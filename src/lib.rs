//! The high level wrapper around the supporting HFF crates.
//!
//! # Examples
//!
//! ```
//! use hff::*;
//!
//! // Creating the content can use the builder:
//! let content = hff([
//!     table("Prime", "Second")
//!     // Metadata and chunks can be pulled from many types of source data.
//!     .metadata("Each table can have metadata.").unwrap()
//!     // Tables can have chunks.
//!     .chunks([
//!         chunk("AChunk", Ecc::INVALID, "Each table can have 0..n chunks of data.").unwrap()
//!     ])
//!     // Tables can have child tables.
//!     .children([
//!         table("Child1", Ecc::INVALID)
//!         .metadata("Unique to this table.").unwrap()
//!         // Chunks can source from many things, in this case it is a PathBuf
//!         // for this file which will be embedded.
//!         .chunks([
//!             chunk("ThisFile", "Copy", std::path::PathBuf::from(file!())).unwrap()
//!         ])
//!     ]),
//!     // And there can be multiple tables at the root.
//!     table("Child2", Ecc::INVALID)
//! ]);
//!
//! // The results can be packaged into an output stream.
//! // This can be anything which supports the std::io::Write trait.
//! let mut buffer = vec![];
//! content.write::<NE>("Test", &mut buffer).unwrap();
//!
//! // Hff can be read back from anything which supports the std::io::Read
//! // trait.  In this case we also read all the data into a cache in memory.
//! // The cache is simply an array with Read+Seek implemented on top of a
//! // Vec<u8>.
//! let (hff, mut cache) = read_stream_full(&mut buffer.as_slice()).unwrap();
//!
//! // The Hff instance contains the structure of the content and can be
//! // iterated in multiple ways.  Here, we'll use the depth first iterator
//! // just to see all the content.
//! for (depth, table) in hff.depth_first() {
//!     // Print information about the table.
//!     println!("{}: {:?} ({})",
//!         depth,
//!         table.primary(),
//!         std::str::from_utf8(&table.metadata(&mut cache).unwrap_or(vec![])).unwrap()
//!     );
//!
//!     // Iterate the chunks.
//!     for chunk in table.chunks() {
//!         println!("{}", std::str::from_utf8(&chunk.data(&mut cache).unwrap()).unwrap());
//!     }
//! }
//! ```
//!
//! # In Progress
//! - [x] Depth first iterator through tables.
//! - [x] More metadata/chunk data source types.  Most things which can be turned into
//! Vec<u8> exist now, read trait for anything which can be immediately pulled in at
//! runtime and finally std::path::{Path, PathBuf} to pull data from a file.
//! - [ ] Yet more metadata/chunk data source types.  Specifically serde and compressed.
//! - [ ] Utility types for metadata.  For instance a simple key=value string map and a
//! simple array of strings.
//! - [x] Change the table builder to allow multiple tables at the 'root' level.
//! Currently the builder expects a single outer table to contain all others.  This
//! is a holdover from a prior format structure which was removed.
//! - [-] After fixing the table builder, implement the lazy header variation so compressed
//! chunks do not have to be stored in memory prior to writing.
//! - [ ] Async-std implementation of the reader.
//! - [ ] Async-std implementation of the writer.
//! - [ ] Tokio implementation of the reader.
//! - [ ] Tokio implementation of the writer.
//! - [ ] Mmap, io_ring and whatever other variations make sense in the long run.
#![warn(missing_docs)]

// Core types.
#[doc(inline)]
pub use hff_core::*;

#[doc(inline)]
pub use hff_std::*;
