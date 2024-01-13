// Pull in core if special behavior is needed.
pub use hff_core;

// Pull in common needs.  Aka: prelude.
pub use hff_core::{
    read::{ChunkView, Hff, TableView},
    utilities,
    write::{chunk, hff, table, ChunkDesc, DataSource, HffDesc, TableBuilder},
    ByteOrder, ChunkCache, ContentInfo, Ecc, Error, Result, Version, BE, LE, NE, OP,
};

mod read;
pub use read::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tests() -> Result<()> {
        let content = hff([
            table("Prime", "Second")
                // Metadata and chunks can be pulled from many types of source data.
                .metadata("Each table can have metadata.")?
                // Tables can have chunks.
                .chunks([chunk(
                    "AChunk",
                    Ecc::INVALID,
                    "Each table can have 0..n chunks of data.",
                )?])
                // Tables can have child tables.
                .children([table("Child1", Ecc::INVALID)
                    .metadata("Unique to this table.")?
                    .chunks([chunk("ThisFile", "Copy", "More data for the chunk.")?])]),
            // And there can be multiple tables at the root.
            table("Child2", Ecc::INVALID),
        ]);

        // Use std variation to write into a vector.
        let mut buffer = vec![];
        use hff_std::Writer;
        content.write::<hff_core::NE>("Test", &mut buffer)?;

        // The reader must take ownership of the given item in order to
        // properly function.
        use std::io::Cursor;
        let reader: Box<dyn ReadSeek> = Box::new(Cursor::new(buffer.into_boxed_slice()));

        // Open the buffer as an hff.
        let hff = open(reader).await?;

        for (depth, table) in hff.depth_first() {
            // Print information about the table.
            println!(
                "{}: {:?} ({})",
                depth,
                table.primary(),
                std::str::from_utf8(hff.read(&table).await?.as_slice()).unwrap()
            );

            // Iterate the chunks.
            for chunk in table.chunks() {
                println!(
                    "{}",
                    std::str::from_utf8(hff.read(&chunk).await?.as_slice()).unwrap()
                );
            }
        }

        Ok(())
    }
}
