//! Implements the basic reader/writer functionality for HFF.
#![warn(missing_docs)]

// Reexport needed types.
#[cfg(feature = "compression")]
pub use hff_core::read::decompress;

// Pull in core if special behavior is needed.
pub use hff_core;

// Pull in common needs.  Aka: prelude.
pub use hff_core::{
    read::{ChunkView, Hff, TableView},
    utilities,
    write::{chunk, hff, table, ChunkDesc, DataSource, HffDesc, TableBuilder},
    ByteOrder, ChunkCache, ContentInfo, Ecc, Error, Result, Version, BE, LE, NE, OP,
};

// Helper traits which provide blanket implementations over the
// required trait combinations.

mod write_seek;
pub use write_seek::WriteSeek;

mod read_seek;
pub use read_seek::ReadSeek;

/// Create a new builder instance.
pub fn build<'a>(
    tables: impl IntoIterator<Item = hff_core::write::TableBuilder<'a>>,
) -> Result<hff_core::write::HffDesc<'a>> {
    Ok(hff_core::write::hff(tables))
}

mod read;
pub use read::*;

mod write;
pub use write::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn test_table<'a>() -> Result<HffDesc<'a>> {
        Ok(hff([
            table("Test", "TestSub")
            .metadata("This is some metadata attached to the table.")?
            .chunks([
                chunk("TRC0", "TRS0", "Chunks can be most types.  This is passed as an arbitrary byte array.".as_bytes())?,
                chunk(
                    "TRC1",
                    "TRS1",
                    "Chunks provided to the table will maintain their order.",
                )?,
                chunk(
                    "TRC2",
                    "TRS2",
                    "So, iterating through the chunks has the same order as presented here.",
                )?,
                chunk(
                    "TRC3",
                    "TRS3",
                    "Chunks can be supplied with data from multiple sources.",
                )?,
                chunk(
                    "TRC4",
                    "TRS4",
                    "In fact, providing a std::path::Path will pull the content of a file in as the chunk data.",
                )?,
                // Compress the string if compression is enabled.
                #[cfg(feature = "compression")]
                chunk(
                    "TRC5",
                    "TRS5",
                    // Compressing chunks is just sending in a tuple with the compression level.
                    // Using lzma for compression and the level is expected to be between 0 and 9.
                    (9, "In the case of a lazy_write, the file will be opened and streamed directly to the writer without being buffered in memory."),
                )?,
                // Don't compress the string if compression is disabled.
                #[cfg(not(feature = "compression"))]
                chunk(
                    "TRC5",
                    "TRS5",
                    "In the case of a lazy_write, the file will be opened and streamed directly to the writer without being buffered in memory.",
                )?,
            ])
            .children([
                table("C0Prime", "C0Sub")
                .metadata("Each table has its own metadata.")?
                .chunks([chunk("C0C0", "C0S0", "Each table also has its own set of chunks.")?])
                .children([
                    table("C1Prime", "C1Sub")
                    .chunks([
                        chunk(
                            "C1C0",
                            "C1S0",
                            "They will only be listed while iterating that specific table.",
                        )?
                    ]),
                    table("C2Prime", "C2Sub")
                    .children([
                        table("C3Prime", "C3Sub")
                        .chunks([
                            chunk("C2C0", "C2S0", "Tables don't *have* to have chunks, tables can be used to simply contain other tables.")?
                        ])
                    ])
                ]),
                table("C4Prime", "C4Sub").chunks([
                    chunk("C4C0", "C4S0","The last chunk in the overall file.")?
                ])
                .metadata("And we're done.")?
            ])
        ]))
    }

    fn checks(hff: &Hff<ChunkCache>) {
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
            assert!(root_children.next().is_none());
        }

        {
            // Check the metadata for the root.
            let root = hff.tables().next().unwrap();
            // The resulting reader is just a reference to the data
            // in the content.  You can take a &mut Read on it if you
            // wish to use it with std::io methods such as copy.
            let metadata = hff.read(&root).unwrap();
            assert!(std::str::from_utf8(metadata)
                .unwrap()
                .starts_with("This is some metadata"));

            // Check the last table (second root child) metadata.
            let mut children = hff.tables().next().unwrap().iter();
            children.next();
            let c4 = children.next().unwrap();
            let metadata = hff.read(&c4).unwrap();
            assert!(std::str::from_utf8(metadata)
                .unwrap()
                .starts_with("And we're done."));
        }

        {
            // Check the root chunks are as expected.
            let root = hff.tables().next().unwrap();

            let test_data = [
                ("TRC0", "TRS0", "Chunks can be most types.  This is passed as an arbitrary byte array."),
                (
                    "TRC1",
                    "TRS1",
                    "Chunks provided to the table will maintain their order.",
                ),
                (
                    "TRC2",
                    "TRS2",
                    "So, iterating through the chunks has the same order as presented here.",
                ),
                (
                    "TRC3",
                    "TRS3",
                    "Chunks can be supplied with data from multiple sources.",
                ),
                (
                    "TRC4",
                    "TRS4",
                    "In fact, providing a std::path::Path will pull the content of a file in as the chunk data.",
                ),
                (
                    "TRC5",
                    "TRS5",
                    "In the case of a lazy_write, the file will be opened and streamed directly to the writer without being buffered in memory.",
                )
            ];
            for (index, chunk) in root.chunks().enumerate() {
                let test_entry = test_data[index];
                assert_eq!(Ecc::new(test_entry.0), chunk.primary());
                assert_eq!(Ecc::new(test_entry.1), chunk.secondary());

                #[cfg(feature = "compression")]
                if chunk.secondary() == Ecc::new("TRS5") {
                    let decompressed = decompress(hff.read(&chunk).unwrap()).unwrap();
                    assert_eq!(decompressed.len(), test_entry.2.len());
                    assert_eq!(decompressed, Vec::from(test_entry.2.as_bytes()));
                } else {
                    assert_eq!(chunk.size(), test_entry.2.len());
                    assert_eq!(
                        hff.read(&chunk).unwrap(),
                        Vec::from(test_entry.2.as_bytes())
                    );
                }
                #[cfg(not(feature = "compression"))]
                {
                    assert_eq!(chunk.size(), test_entry.2.len());
                    assert_eq!(
                        hff.read(&chunk).unwrap(),
                        Vec::from(test_entry.2.as_bytes())
                    );
                }
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
        use std::io::Seek;

        // Simple dev test for structure.
        {
            let content = test_table().unwrap();
            let buffer = vec![];
            let mut writer = std::io::Cursor::new(buffer);
            assert!(content
                .lazy_write::<hff_core::NE>("Test", &mut writer)
                .is_ok());

            // Read it back in and iterate.
            writer.rewind().unwrap();
            let access = crate::read::read(&mut writer).unwrap();
            checks(&access);
        }

        // Simple dev test for structure.
        {
            let content = test_table().unwrap();
            let mut buffer = vec![];

            assert!(content.write::<hff_core::OP>("Test", &mut buffer).is_ok());

            // Read it back in and iterate.
            let access = crate::read::read(&mut buffer.as_slice()).unwrap();
            checks(&access);
        }
    }
}
