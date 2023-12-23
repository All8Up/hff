use crate::{DataSource, Error, Result};
use hff_core::Ecc;

mod table_array;
pub(crate) use table_array::TableArray;

mod chunk_array;
pub(crate) use chunk_array::ChunkArray;

mod data_array;
pub(crate) use data_array::DataArray;

mod chunk_desc;
pub(crate) use chunk_desc::ChunkDesc;

mod table_desc;
pub(crate) use table_desc::TableDesc;

mod table_builder;
pub(crate) use table_builder::TableBuilder;

mod hff_content;
pub use hff_content::HffContent;

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
pub fn hff<'a>(tables: impl IntoIterator<Item = TableBuilder<'a>>) -> HffContent<'a> {
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
    HffContent::new(table_array, chunk_array, data_array)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let content = hff([
            table("p0", "s0")
                .metadata("123")
                .unwrap()
                .children([table("p1", "s1")
                    .metadata("1234")
                    .unwrap()
                    .chunks([
                        chunk("c0", "cs0", "chunk 0").unwrap(),
                        chunk("c1", "cs1", "chunk 1").unwrap(),
                        chunk("c2", "cs2", "chunk 2").unwrap(),
                    ])
                    .children([
                        table("p2", "s2").metadata("12345").unwrap().chunks([]),
                        table("p3", "s3").metadata("123456").unwrap().chunks([]),
                    ])])
                .chunks([]),
            table("p4", "s4").metadata("1234567").unwrap(),
            table("p5", "s5")
                .metadata("12345678")
                .unwrap()
                .chunks([chunk("c3", "cs3", "chunk 3").unwrap()]),
        ]);

        let mut buffer = vec![];
        content.write::<hff_core::LE>("Test", &mut buffer).unwrap();

        let (hff, _cache) = crate::read_stream_full(&mut buffer.as_slice()).unwrap();
        println!("{:#?}", hff);
        println!("-----------------------------");
        for (depth, table) in hff.depth_first() {
            println!("-- <{}>: <{:?}>", depth, table);
        }
        println!("-----------------------------");

        //assert!(false);
    }
}
