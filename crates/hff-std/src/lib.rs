//! Implements the basic reader/writer functionality for HFF.
#![warn(missing_docs)]
use hff_core::{Ecc, Error, Result};

mod data_source;
pub use data_source::DataSource;

mod chunk_desc;
pub use chunk_desc::ChunkDesc;

mod table_desc;
pub use table_desc::TableDesc;

mod table_builder;
pub use table_builder::TableBuilder;

mod data_builder;
pub use data_builder::DataBuilder;

mod writer;
pub use writer::write_stream;

mod reader;
pub use reader::read_stream;

/// Create a table structure to be contained in the HFF.
/// Panics if the given Ecc data is invalid.
pub fn table(primary: impl Into<Ecc>, secondary: impl Into<Ecc>) -> TableBuilder {
    TableBuilder::new(primary.into(), secondary.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_table() -> Result<TableDesc> {
        Ok(table("Test", "TestSub")
            .metadata("This is some metadata attached to the table.")?
            .chunk("TRC0", "TRS0", "Chunks can be most types.")?
            .chunk(
                "TRC1",
                "TRS1",
                "Both chunks and tables will maintain their order within the file.",
            )?
            .chunk(
                "TRC2",
                "TRS2",
                "But, there is no relationship maintained between chunks and tables.",
            )?
            .chunk(
                "TRC3",
                "TRS3",
                "In other words, we are creating two lists, one for chunks and one for tables.",
            )?
            .chunk(
                "TRC4",
                "TRS4",
                "So, the order of adding chunks inbetween tables has no impact on the result.",
            )?
            .chunk(
                "TRC5",
                "TRS5",
                "Only which table the chunk is associated with will matter..",
            )?
            .table(
                table("C0Prime", "C0Sub")
                    .metadata("Each table has its own metadata.")?
                    .chunk("C0C0", "C0S0", "Each table also has its own set of chunks.")?
                    .table(
                        table("C1Prime", "C1Sub")
                            .chunk(
                                "C1C0",
                                "C1S0",
                                "They will only be listed while iterating that specific table.",
                            )?
                            .end(),
                    )
                    .table(
                        table("C2Prime", "C2Sub")
                            .table(table("C3Prime", "C3Sub")
                            .chunk("C2C0", "C2S0", "Tables don't *have* to have chunks, tables can be used to simply contain other tables.")?
                            .end())
                            .end(),
                    )
                    .end(),
            )
            .table(table("C4Prime", "C4Sub").chunk("C4C0", "C4S0","The last chunk in the overall file.")?.metadata("And we're done.")?.end())
            .end())
    }

    #[test]
    fn test() {
        // Simple dev test for structure.
        {
            let table = test_table().unwrap();
            let mut buffer = vec![];

            assert!(write_stream::<hff_core::NE>("Test", table, &mut buffer).is_ok());
            //assert!(read_stream(&mut buffer.as_slice()).is_ok());
        }

        // {
        //     let table = test_table();
        //     let mut buffer = vec![];

        //     assert!(write_stream::<hff_core::OP>("Test", table, &mut buffer).is_ok());
        //     assert!(read_stream(&mut buffer.as_slice()).is_ok());
        // }

        assert!(false);
    }
}
