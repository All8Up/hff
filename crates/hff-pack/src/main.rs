mod options;
use normpath::PathExt;
use options::Options;

mod structure;
use structure::Structure;

use hff_std::{hff, Ecc, Result, Writer, NE};

use clap::Parser;

/// An archive entry.
pub const HFF_ARCHIVE: Ecc = Ecc::new("_ARCHIVE");
/// This is the type for a directory table entry.
pub const HFF_DIR: Ecc = Ecc::new("_DIR");
/// This is the type for a file chunk.
pub const HFF_FILE: Ecc = Ecc::new("_FILE");
/// If the chunk is compressed.
pub const HFF_LZMA: Ecc = Ecc::new("_LZMA");

#[async_std::main]
async fn main() -> Result<()> {
    // Parse the command line options.
    let options = Options::parse();
    println!("{:#?}", options);

    // Either a script or an input directory must be provided.
    if options.input.is_dir() {
        // Create a file to write the content into.
        let mut output = std::fs::File::create(&options.output)?;

        // Get the input file and the parent path.
        let input = options.input.normalize()?;
        let input = input.as_path();
        let parent = input.parent().unwrap().into();

        // Scan the input and then strip the parent path off of it.
        let structure = Structure::new(input.into(), !options.non_recursive).await?;
        let structure = structure.strip_prefix(parent)?;

        // Build up the tables for the structure.
        let tables = structure.to_tables(parent)?;

        // Write and compress.

        let desc = hff([]);

        let mut buffer = vec![];
        desc.write::<NE>(HFF_ARCHIVE, &mut buffer)?;
    } else if options.input.is_file() {
        println!("The input must be a directory.");
        std::process::exit(-1);
    } else {
        println!("The input directory does not exist.");
        std::process::exit(-1);
    }

    Ok(())
}
