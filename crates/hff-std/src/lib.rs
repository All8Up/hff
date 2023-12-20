//! Implements the basic reader/writer functionality for HFF.
#![warn(missing_docs)]
use hff_core::{Ecc, Error, Result};

mod data_source;
pub use data_source::{DataSource, StdWriter};

mod writer;
pub use writer::{write_stream, TableBuilder, TableDesc};

mod reader;
pub use reader::{
    read_stream, read_stream_full, ChunkCache, ChunkIter, ChunkView, DepthFirstIter, Hff, ReadSeek,
    TableIter, TableView,
};

/// Create a table structure to be contained in the HFF.
/// Panics if the given Ecc data is invalid.
pub fn table(primary: impl Into<Ecc>, secondary: impl Into<Ecc>) -> TableBuilder {
    TableBuilder::new(primary.into(), secondary.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::{ChunkCache, Hff};

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
            .table(table("C4Prime", "C4Sub").chunk("C4C0", "C4S0","The last chunk in the overall file.")?
                .metadata("And we're done.")?.end())
            .end())
    }

    fn checks(hff: &Hff, cache: &mut ChunkCache) {
        {
            // Check the content of root is as expected.
            let root = hff.tables().next().unwrap();
            assert_eq!(root.primary(), "Test".into());
            assert_eq!(root.secondary(), "TestSub".into());
            assert_eq!(root.child_count(), 2);
            assert_eq!(root.chunk_count(), 6);

            // Check that we get a proper child iterator from the root.
            let mut root_children = root.iter();
            let c0 = root_children.next().unwrap();
            assert_eq!(c0.primary(), "C0Prime".into());
            let c4 = root_children.next().unwrap();
            assert_eq!(c4.primary(), "C4Prime".into());
            assert_eq!(root_children.next(), None);
        }

        {
            // Check the metadata for the root.
            let root = hff.tables().next().unwrap();
            let metadata = root.metadata(cache).unwrap();
            assert!(std::str::from_utf8(&metadata)
                .unwrap()
                .starts_with("This is some metadata"));

            // Check the last table (second root child) metadata.
            let mut children = hff.tables().next().unwrap().iter();
            children.next();
            let c4 = children.next().unwrap();
            let metadata = c4.metadata(cache).unwrap();
            assert!(std::str::from_utf8(&metadata)
                .unwrap()
                .starts_with("And we're done."));
        }

        {
            // Check the root chunks are as expected.
            let root = hff.tables().next().unwrap();

            let test_data = [
                ("TRC0", "TRS0", "Chunks can be most types."),
                (
                    "TRC1",
                    "TRS1",
                    "Both chunks and tables will maintain their order within the file.",
                ),
                (
                    "TRC2",
                    "TRS2",
                    "But, there is no relationship maintained between chunks and tables.",
                ),
                (
                    "TRC3",
                    "TRS3",
                    "In other words, we are creating two lists, one for chunks and one for tables.",
                ),
                (
                    "TRC4",
                    "TRS4",
                    "So, the order of adding chunks inbetween tables has no impact on the result.",
                ),
                (
                    "TRC5",
                    "TRS5",
                    "Only which table the chunk is associated with will matter..",
                ),
            ];
            for (index, chunk) in root.chunks().enumerate() {
                let test_entry = test_data[index];
                assert_eq!(Ecc::new(test_entry.0), chunk.primary());
                assert_eq!(Ecc::new(test_entry.1), chunk.secondary());
                assert_eq!(chunk.size(), test_entry.2.len());
                assert_eq!(
                    chunk.data(cache).unwrap(),
                    Vec::from(test_entry.2.as_bytes())
                );
            }

            {
                let test_data = [
                    (0, "Test", "TestSub"),
                    (1, "C0Prime", "C0Sub"),
                    (2, "C1Prime", "C1Sub"),
                    (2, "C2Prime", "C2Sub"),
                    (3, "C3Prime", "C3Sub"),
                    (1, "C4Prime", "C4Sub"),
                ];
                // Test depth first iteration.
                for ((depth, table), data) in hff.depth_first().zip(test_data.iter()) {
                    assert_eq!(depth, data.0);
                    assert_eq!(table.primary(), data.1.into());
                    assert_eq!(table.secondary(), data.2.into());
                }
            }
        }
    }

    #[test]
    fn test() {
        // Simple dev test for structure.
        {
            let table = test_table().unwrap();
            let mut buffer = vec![];

            assert!(write_stream::<hff_core::NE>("Test", table, &mut buffer).is_ok());

            // Read it back in and iterate.
            let (hff, mut cache) = read_stream_full(&mut buffer.as_slice()).unwrap();
            checks(&hff, &mut cache);
        }

        // Simple dev test for structure.
        {
            let table = test_table().unwrap();
            let mut buffer = vec![];

            assert!(write_stream::<hff_core::OP>("Test", table, &mut buffer).is_ok());

            // Read it back in and iterate.
            let (hff, mut cache) = read_stream_full(&mut buffer.as_slice()).unwrap();
            checks(&hff, &mut cache);
        }
    }
}
