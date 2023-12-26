mod read;
pub use read::*;

#[cfg(test)]
mod tests {
    use super::*;
    use hff_core::{read::Hff, write::*, Ecc, Result};
    use hff_std::{Chunk, Metadata};

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
                    // Chunks can source from many things, in this case it is a PathBuf
                    // for this file which will be embedded.
                    .chunks([chunk(
                        "ThisFile",
                        "Copy",
                        std::path::PathBuf::from(file!()),
                    )?])]),
            // And there can be multiple tables at the root.
            table("Child2", Ecc::INVALID),
        ]);

        // Use std variation to write into a vector.
        let mut buffer = vec![];
        use hff_std::Writer;
        content.write::<hff_core::NE>("Test", &mut buffer)?;

        // Use async_std to read from the buffer.
        let (hff, mut cache) = Hff::read_full(&mut buffer.as_slice()).await?;

        for (depth, table) in hff.depth_first() {
            // Print information about the table.
            println!(
                "{}: {:?} ({})",
                depth,
                table.primary(),
                std::str::from_utf8(&table.metadata(&mut cache).unwrap_or(vec![])).unwrap()
            );

            // Iterate the chunks.
            for chunk in table.chunks() {
                println!(
                    "{}",
                    std::str::from_utf8(&chunk.read(&mut cache).unwrap()).unwrap()
                );
            }
        }

        Ok(())
    }
}
