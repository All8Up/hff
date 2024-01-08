use super::Result;
use clap::Args;
use hff_std::{hff, Ecc, Writer, BE, LE, NE};
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
    pub compression: Option<u32>,

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

        // Scan the structure of the input.
        let structure = Structure::new(&self.input, self.recurse)?;
        let parent = self.input.parent().unwrap();
        let structure = structure.strip_prefix(parent)?;

        // Build up the tables for the structure.
        let root = structure.to_tables::<NE>(parent, |_| self.compression)?;

        // Create a file to write the content into.
        let mut output = File::create(&self.output)?;
        // And write in the selected endian or native.
        if !(self.little_endian || self.big_endian) {
            hff([root]).write::<NE>(HFF_ARCHIVE, &mut output)?;
        } else {
            if self.big_endian {
                hff([root]).write::<BE>(HFF_ARCHIVE, &mut output)?;
            } else if self.little_endian {
                hff([root]).write::<LE>(HFF_ARCHIVE, &mut output)?;
            }
        }
        output.flush()?;

        Ok(())
    }
}
