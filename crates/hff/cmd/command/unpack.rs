use super::Result;
use clap::Args;
use hff_core::{read::TableIter, utilities::Hierarchical, Error};
use hff_std::{open, ContentInfo, Hff, StdReader, TableView};
use log::trace;
use std::{
    fs::{create_dir_all, File},
    io::Read,
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
                if first.identifier().as_eccu64().0 == super::HFF_DIR {
                    trace!("Found an archived directory.");
                    self.unpack_directory(hff)
                } else if first.identifier().as_eccu64().0 == super::HFF_FILE {
                    trace!("Found an archived file.");
                    self.unpack_file(hff)
                } else if first.identifier().as_eccu64().0 == super::HFF_EMBEDDED {
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

            // There can be only one top level if there are multiple files.
            if hff.tables().count() == 1 {
                // This is a single directory at the top.
                // Just write out all the chunks to the output location.
                let table = hff.tables().next().unwrap();
                let hierarchy = Hierarchical::from_bytes(hff.get(&table)?.as_slice())?;
                self.write_chunks(&hff, &self.output, hierarchy.content(), &table)?;
                self.unpack_level(0, &self.output, &hff, table.iter(), hierarchy.children())?;
                Ok(())
            } else {
                Err(Error::Invalid(format!("Invalid structure.")))
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
        hierarchy: &[Hierarchical],
    ) -> Result<()> {
        // Iterate the tables.
        for (table, desc) in level.zip(hierarchy.iter()) {
            let dir = desc.key();
            let names = desc.content();

            // Create a directory for the child.
            let child_dir = location.join(dir);
            trace!("Child: {:?}", child_dir);
            create_dir_all(&child_dir)?;

            // If there are chunks at this level, write them.
            self.write_chunks(hff, &child_dir, names, &table)?;

            // Recurse into the child.
            self.unpack_level(depth + 1, &child_dir, hff, table.iter(), desc.children())?;
        }
        Ok(())
    }

    /// Write chunks.
    fn write_chunks(
        &self,
        hff: &Hff<StdReader>,
        path: &Path,
        names: &[String],
        table: &TableView<'_, StdReader>,
    ) -> Result<()> {
        for (index, chunk) in table.chunks().into_iter().enumerate() {
            let (primary, uncompressed_size) = chunk.identifier().as_eccu64();
            if primary == super::HFF_FILE {
                if uncompressed_size > 0 {
                    // Read from the chunk.
                    let reader: &mut dyn Read = &mut *hff.read(&chunk)?;
                    let mut buffer = vec![0; chunk.len() as usize];
                    reader.read_exact(&mut buffer)?;

                    // Create a decompressor for the chunk.
                    let buffer = hff_std::decompress(&mut buffer.as_mut_slice())?;

                    let mut output = File::create(path.join(&names[index]))?;
                    std::io::copy(&mut buffer.as_slice(), &mut output)?;
                } else {
                    let reader: &mut dyn Read = &mut *hff.read(&chunk)?;
                    let mut output = File::create(path.join(&names[index]))?;
                    std::io::copy(reader, &mut output)?;
                }
            } else {
                unimplemented!()
            }
        }
        Ok(())
    }

    /// Unpack a hff file.
    fn unpack_file(&self, _hff: Hff<StdReader>) -> Result<()> {
        Ok(())
    }

    /// Unpack a hff file that has been singularly archived.
    fn unpack_hff(&self, _hff: Hff<StdReader>) -> Result<()> {
        Ok(())
    }
}
