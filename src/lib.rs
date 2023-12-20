//! The high level wrapper around the supporting HFF crates.
//!
//! # Examples (See[^note])
//!
//! ```
//! use hff::*;
//!
//! // Creating the content can use the table builder:
//! let content = table("Prime", "Second")
//!     .metadata("Each table can have metadata.").unwrap()
//!     .chunk("AChunk", Ecc::INVALID, "Each table can have 0..n chunks of data.").unwrap()
//!     .table(table("Child1", Ecc::INVALID).end())
//!     .table(table("Child2", Ecc::INVALID).end())
//!     .end();
//!
//! // The results can be packaged into an output stream (std::io::Write)
//! // This can be anything which supports the std::io::Write trait.
//! let mut buffer = vec![];
//! write_stream::<NE>("HffFile", content, &mut buffer).unwrap();
//!
//! // Hff can be read back from anything which supports the std::io::Read
//! // trait.  
//! let (hff, mut cache) = read_stream_full(&mut buffer.as_slice()).unwrap();
//!
//! // Note that hff is const and represents just the structure of the file, it does
//! // not contain any of the data, not even the metadata attached to tables.
//! // In order to fetch the content, you can use the iterators:
//! for table in hff.tables() {
//!     // Will only iterate over the outer table(s).  (In this case the
//!     // "Prime"|"Second" table.)
//!     
//!     // Iterate the chunks in this table.
//!     for chunk in table.chunks() {
//!         // Read the chunk data.
//!         let _data = chunk.data(&mut cache).unwrap();
//!     }
//! }
//! ```
//!
//! # In Progress
//! - [ ] Depth first iterator through tables.
//! - [ ] More metadata/chunk data source types.  Currently only the &str data type
//! is implemented for initial testing purposes.
//! - [ ] Change the table builder to allow multiple tables at the 'root' level.
//! Currently the builder expects a single outer table to contain all others.  This
//! is a holdover from a prior format structure which was removed.
//! - [ ] Async-std implementation of the reader.
//! - [ ] Async-std implementation of the writer.
//! - [ ] Tokio implementation of the reader.
//! - [ ] Tokio implementation of the writer.
//! - [ ] Mmap, io_ring and whatever other variations make sense in the long run.
//!
//! [^note]: Currently chunks can only contain strings, this will be expanded as soon
//! as the major structure is stable.
#![warn(missing_docs)]

// Core types.
#[doc(inline)]
pub use hff_core::*;

#[doc(inline)]
pub use hff_std::*;
