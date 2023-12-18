//! Implements the basic reader/writer functionality for HFF.
#![warn(missing_docs)]
use std::mem::size_of;

use hff_core::{Ecc, Header, Result, Table, NE};

mod chunk_desc;
pub use chunk_desc::ChunkDesc;

mod table_desc;
pub use table_desc::TableDesc;

mod table_builder;
pub use table_builder::TableBuilder;

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

    fn test_table() -> TableDesc {
        table("Test", "TestSub")
            .table(
                table("C0Prime", "C0Sub")
                    .table(table("C1Prime", "C1Sub").end())
                    .table(
                        table("C2Prime", "C2Sub")
                            // .table(table("C3Prime", "C3Sub").end())
                            .end(),
                    )
                    .end(),
            )
            .table(table("C4Prime", "C4Sub").end())
            .end()
    }

    #[test]
    fn test() {
        // Simple dev test for structure.
        {
            let table = test_table();
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
