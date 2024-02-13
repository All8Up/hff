use super::Result;
use clap::Args;
use hff_std::{hff, Ecc, IdType, Writer, BE, LE, NE};
use log::trace;
use normpath::PathExt;
use std::{fs::File, io::Write, path::PathBuf};

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

/// Package a file or directory into an hff container.
#[derive(Debug, Args)]
pub struct Pack {
    /// The input path to a file or directory.
    pub input: PathBuf,
    /// The output file to pack to.
    pub output: PathBuf,

    /// If we want to be recursive.
    #[arg(short, long, overrides_with = "_no_recurse")]
    pub recurse: bool,
    /// Override the default recursive behavior.
    #[arg(long = "no-recurse", overrides_with = "recurse")]
    pub _no_recurse: bool,

    /// Compression level for the content?
    #[arg(long, short)]
    pub compress: Option<u32>,

    /// Force big endian mode.
    #[arg(long, conflicts_with = "little_endian")]
    pub big_endian: bool,
    /// Force little endian mode.
    #[arg(long, conflicts_with = "big_endian")]
    pub little_endian: bool,
}

impl Pack {
    /// Execute the subcommand.
    pub fn execute(self) -> Result<()> {
        use super::Structure;
        let input = self.input.normalize()?;
        let input: std::path::PathBuf = input.into();

        // Scan the structure of the input.
        let structure = Structure::new(&input, self.recurse)?;
        let parent = input.parent().unwrap();
        trace!("Input: {:?}", input);
        trace!("Parent: {:?}", parent);
        let structure = structure.strip_prefix(parent)?;

        // Build up the tables for the structure.
        let root = structure.to_tables::<NE>(parent, |_| self.compress)?;

        // Create a file to write the content into.
        let mut output = File::create(&self.output)?;
        // And write in the selected endian or native.
        if !(self.little_endian || self.big_endian) {
            hff([root]).write::<NE>(IdType::EccU64, HFF_ARCHIVE, &mut output)?;
        } else {
            if self.big_endian {
                hff([root]).write::<BE>(IdType::EccU64, HFF_ARCHIVE, &mut output)?;
            } else if self.little_endian {
                hff([root]).write::<LE>(IdType::EccU64, HFF_ARCHIVE, &mut output)?;
            }
        }
        output.flush()?;

        Ok(())
    }
}
