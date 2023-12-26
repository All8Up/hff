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

use crate::{Ecc, Error, Result};

/// Start building a new table.
pub fn table<'a>(primary: impl Into<Ecc>, secondary: impl Into<Ecc>) -> TableBuilder<'a> {
    TableBuilder::new(primary.into(), secondary.into())
}

/// Build a new chunk.
pub fn chunk<'a, T>(
    primary: impl Into<Ecc>,
    secondary: impl Into<Ecc>,
    content: T,
) -> Result<ChunkDesc<'a>>
where
    T: TryInto<DataSource<'a>>,
    <T as TryInto<DataSource<'a>>>::Error: std::fmt::Debug,
    Error: From<<T as TryInto<DataSource<'a>>>::Error>,
{
    Ok(ChunkDesc::new(
        primary.into(),
        secondary.into(),
        content.try_into()?,
    ))
}

/// Build the structure of the Hff content.
pub fn hff<'a>(tables: impl IntoIterator<Item = TableBuilder<'a>>) -> HffDesc<'a> {
    // Split the tables into their components.
    let mut table_array = TableArray::new();
    let mut chunk_array = ChunkArray::new();
    let mut data_array = DataArray::new();

    // Collect the tables into a vector so we know the length.
    let tables = tables
        .into_iter()
        .map(|desc| desc.finish())
        .collect::<Vec<_>>();

    let table_count = tables.len();
    for (index, table) in tables.into_iter().enumerate() {
        // Determine if this table has a sibling.
        let has_sibling = index < table_count - 1;
        // And flatten the table.
        table.flatten(
            has_sibling,
            &mut table_array,
            &mut chunk_array,
            &mut data_array,
        );
    }
    HffDesc::new(table_array, chunk_array, data_array)
}
