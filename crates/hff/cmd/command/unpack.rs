use super::Result;
use clap::Args;
use hff_core::{read::TableIter, utilities::Ksv, Error};
use hff_std::{open, ContentInfo, Hff, StdReader, TableView};
use log::trace;
use normpath::PathExt;
use std::{
    fs::{create_dir_all, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

/// Unpack an archive hff to the given location or file.
#[derive(Debug, Args)]
pub struct Unpack {
    /// The input hff to unpack.
    pub input: PathBuf,
    /// The output location for the command.
    pub output: PathBuf,
}

impl Unpack {
    /// Execute the subcommand.
    pub fn execute(self) -> Result<()> {
        // Open and validate the hff file.
        let hff = open(File::open(&self.input)?)?;
        if hff.content_type() == super::HFF_ARCHIVE {
            // Figure out if the archive is a single file
            // or a directory.
            if let Some((_, first)) = hff.depth_first().next() {
                if first.primary() == super::HFF_DIR {
                    trace!("Found an archived directory.");
                    self.unpack_directory(hff)
                } else if first.primary() == super::HFF_FILE {
                    trace!("Found an archived file.");
                    self.unpack_file(hff)
                } else if first.primary() == super::HFF_EMBEDDED {
                    trace!("Found an archived embedded hff.");
                    self.unpack_hff(hff)
                } else {
                    Err(Error::Invalid(
                        "Invalid archive, unknown first table type.".into(),
                    ))
                }
            } else {
                Err(Error::Invalid("Invalid archive, no tables.".into()))
            }
        } else {
            Err(Error::Invalid(format!(
                "Invalid input, not an archive: {}",
                self.output.display()
            )))
        }
    }

    /// Unpack a directory archive.
    fn unpack_directory(&self, hff: Hff<StdReader>) -> Result<()> {
        // The output must not exist or be a directory that we can
        // store the data within.
        if self.output.is_dir() || !self.output.exists() {
            // It is a valid output location.
            // Make sure it exists and then proceed.
            create_dir_all(&self.output)?;

            // If the top level of the tables is a single table,
            // we are going to ignore it's name and just dump the
            // files and then recurse into the children.  Otherwise
            // we'll be using all the top level entries.
            if hff.tables().count() == 1 {
                // This is a single directory at the top.
                // Just write out all the chunks to the output location.
                self.unpack_level(0, &self.output, &hff, hff.tables());
                Ok(())
            } else {
                // There are multiple top level entries, we'll
                // create them as child directories and unpack the content
                // there.
                Ok(())
            }
        } else {
            Err(Error::Invalid(format!("Output invalid: {:?}", self.output)))
        }
    }

    /// Unpack a single table.
    fn unpack_level(
        &self,
        depth: usize,
        location: &Path,
        hff: &Hff<StdReader>,
        level: TableIter<'_, StdReader>,
    ) -> Result<()> {
        // Iterate the tables.
        for table in level {
            // Get the metadata as a Ksv.
            let data = Ksv::from_bytes(hff.get(&table)?.as_slice())?;
            let dir = &data["dir"][0];
            let names = &data["files"];

            // Create a directory for the child.
            let child_dir = location.join(dir);
            trace!("Child: {:?}", child_dir);
            create_dir_all(&child_dir)?;

            // If there are chunks at this level, output them.
            for (index, chunk) in table.chunks().into_iter().enumerate() {
                if chunk.secondary() == super::HFF_LZMA {
                    // Read from the chunk.
                    let reader: &mut dyn Read = &mut *hff.read(&chunk)?;
                    let mut buffer = vec![0; chunk.len() as usize];
                    reader.read_exact(&mut buffer)?;

                    // Create a decompressor for the chunk.
                    let mut buffer = hff_std::decompress(&mut buffer.as_mut_slice())?;

                    let mut output = File::create(child_dir.join(&names[index]))?;
                    std::io::copy(&mut buffer.as_slice(), &mut output)?;
                } else {
                    let reader: &mut dyn Read = &mut *hff.read(&chunk)?;
                    let mut output = File::create(child_dir.join(&names[index]))?;
                    std::io::copy(reader, &mut output)?;
                }
            }

            // Recurse into the child.
            self.unpack_level(depth + 1, &child_dir, hff, table.iter())?;
        }
        Ok(())
    }

    /// Unpack a hff file.
    fn unpack_file(&self, hff: Hff<StdReader>) -> Result<()> {
        Ok(())
    }

    /// Unpack a hff file that has been singularly archived.
    fn unpack_hff(&self, hff: Hff<StdReader>) -> Result<()> {
        Ok(())
    }
}
