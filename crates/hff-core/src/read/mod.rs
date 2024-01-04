//! Higher level wrapping over the structure in order to support
//! reading hff files.

mod inspection;
pub use inspection::Inspection;

mod chunk_view;
pub use chunk_view::ChunkView;

mod table_view;
pub use table_view::TableView;

mod depth_first_iter;
pub use depth_first_iter::DepthFirstIter;

mod table_iter;
pub use table_iter::TableIter;

mod chunk_iter;
pub use chunk_iter::ChunkIter;

#[cfg(feature = "compression")]
mod decompress;
#[cfg(feature = "compression")]
pub use decompress::decompress;

mod hff;
pub use hff::Hff;

/// Iteration data attached to the Hff for varying access needs.
pub trait IterData: std::fmt::Debug + Copy + Clone + Default {}

// Default iterdata is nothing.
impl IterData for () {}
