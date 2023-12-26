//! Higher level support for writing hff files.

mod chunk_array;
pub use chunk_array::ChunkArray;

mod data_source;
pub use data_source::DataSource;

mod table_array;
pub use table_array::TableArray;

mod data_array;
pub use data_array::DataArray;

mod chunk_desc;
pub use chunk_desc::ChunkDesc;

mod table_builder;
pub use table_builder::TableBuilder;

mod table_desc;
pub use table_desc::TableDesc;

mod hff_desc;
pub use hff_desc::HffDesc;
