mod options;
use std::io::Write;

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
/// This is the type for a regular file chunk.
pub const HFF_FILE: Ecc = Ecc::new("_FILE");
/// This is the type for a Hff file which is embedded into the archive.
pub const HFF_EMBEDDED: Ecc = Ecc::new("_HFF");
/// If the chunk is compressed.
pub const HFF_LZMA: Ecc = Ecc::new("_LZMA");

#[async_std::main]
async fn main<'a>() -> Result<()> {
    // Parse the command line options.
    let options = Options::parse();

    println!("{:#?}", options);

    // Either a script or an input directory must be provided.
    if options.input.is_dir() {
        // Get the input file and the parent path.
        let input = options.input.normalize()?;
        let input = input.as_path();
        let parent = input.parent().unwrap().into();

        // Scan the input and then strip the parent path off of it.
        let structure = Structure::new(input.into(), !options.non_recursive).await?;
        let structure = structure.strip_prefix(parent)?;

        // Build up the tables for the structure.
        let tables = structure.to_tables::<NE>(parent, |_| None)?;

        // Create a file to write the content into.
        let mut output = std::fs::File::create(&options.output)?;
        // And write.
        hff(tables.into_iter()).write::<NE>(HFF_ARCHIVE, &mut output)?;
        output.flush()?;
    } else if options.input.is_file() {
        // Get the input file and the parent path.
        let input = options.input.normalize()?;
        let input = input.as_path();
        let parent = input.parent().unwrap().into();

        let structure = Structure::new(input.into(), !options.non_recursive).await?;
        let structure = structure.strip_prefix(parent)?;

        // Build up the tables for the structure.
        let tables = structure.to_tables::<NE>(parent, |_| options.compression)?;

        // Create a file to write the content into.
        let mut output = std::fs::File::create(&options.output)?;
        // And write.
        hff(tables.into_iter()).write::<NE>(HFF_ARCHIVE, &mut output)?;
        output.flush()?;
    } else {
        println!("The input directory does not exist.");
        std::process::exit(-1);
    }

    Ok(())
}
