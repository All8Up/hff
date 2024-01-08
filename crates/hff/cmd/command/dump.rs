use clap::Args;
use hff_std::*;
use std::path::PathBuf;

/// Dump out the structure of an hff file.
#[derive(Debug, Args)]
pub struct Dump {
    /// The input file.
    pub input: PathBuf,
    /// Indent the information by depth?
    #[arg(long, overrides_with = "_no_indent")]
    pub indent: bool,
    /// No indention.
    #[arg(long = "no-indent")]
    pub _no_indent: bool,
    /// Max length to indent.
    #[arg(long, default_value = "20")]
    pub max_indent: usize,
    /// Number of spaces per indent level to use.
    #[arg(long, default_value = "2")]
    pub indent_size: usize,
    /// Dump chunk type, sub-type?
    #[arg(long, default_value = "false")]
    pub chunk_types: bool,
    /// Dump metadata?
    #[arg(long, default_value = "false")]
    pub metadata: bool,
    /// Interpret the metadata as a key string vector table.
    #[arg(long, default_value = "false", conflicts_with = "as_string_vec")]
    pub as_ksv: bool,
    /// Interpret the metadata as a string vector.
    #[arg(long, conflicts_with = "as_ksv")]
    pub as_string_vec: bool,
}

impl Dump {
    /// Execute the dump subcommand.
    pub fn execute(self) -> Result<()> {
        use std::fs::File;

        // The input must exist and be a single file.
        if self.input.exists() && self.input.metadata()?.is_file() {
            // Open the hff and check it is valid.
            let hff = open(File::open(&self.input)?)?;

            // Iterate through the content.
            println!();
            println!("----------");
            for (depth, table) in hff.depth_first() {
                self.dump(&hff, depth, &table)?;
            }
            println!("----------");

            Ok(())
        } else {
            Err(Error::Invalid(format!(
                "Invalid input: {}",
                self.input.display().to_string()
            )))
        }
    }

    /// Dump information about the provided table.
    fn dump(
        &self,
        hff: &Hff<StdReader>,
        depth: usize,
        table: &TableView<'_, StdReader>,
    ) -> Result<()> {
        // Always print out the table information.
        println!(
            "{} ({:<8} | {:<8} : children: {} chunks: {})",
            self.indent(depth),
            table.primary().to_string(),
            table.secondary().to_string(),
            table.child_count(),
            table.chunk_count()
        );

        // Print out the metadata if desired.
        if self.metadata {
            self.dump_metadata(hff, depth, table)?;
        }

        // Print out chunk types if desired.
        if self.chunk_types {
            self.dump_chunk_types(depth, table)?;
        }

        Ok(())
    }

    /// Dump out the metadata if any.
    fn dump_metadata(
        &self,
        hff: &Hff<StdReader>,
        depth: usize,
        table: &TableView<'_, StdReader>,
    ) -> Result<()> {
        let metadata = hff.get(table)?;
        if self.as_ksv {
            match hff_std::hff_core::utilities::Ksv::from_bytes(metadata.as_slice()) {
                Ok(ksv) => {
                    println!(" {} {:#?}", self.indent(depth), ksv);
                }
                Err(_) => {
                    println!(" {} <Not a key string vector>", self.indent(depth));
                }
            }
        } else if self.as_string_vec {
            match hff_std::hff_core::utilities::StringVec::from_bytes(metadata.as_slice()) {
                Ok(sv) => {
                    println!(" {} {:#?}", self.indent(depth), sv);
                }
                Err(_) => {
                    println!(" {} <Not a string vector>", self.indent(depth));
                }
            }
        } else {
            match std::str::from_utf8(&metadata) {
                Ok(s) => {
                    println!(" {}{}", self.indent(depth), s);
                }
                Err(_) => {
                    println!(
                        " {} ({:<8} {:<8})",
                        self.indent(depth),
                        metadata.len(),
                        table.offset()
                    );
                }
            }
        }

        Ok(())
    }

    /// Dump out the chunk types if any.
    fn dump_chunk_types(&self, depth: usize, table: &TableView<'_, StdReader>) -> Result<()> {
        for chunk in table.chunks() {
            println!(
                " {} [{:<8} | {:<8} Len: {}]",
                self.indent(depth),
                chunk.primary().to_string(),
                chunk.secondary().to_string(),
                chunk.len()
            );
        }

        Ok(())
    }

    /// Get a string of spaces representing the indent level desired.
    fn indent(&self, depth: usize) -> String {
        if self.indent {
            if self.indent_size * depth < self.max_indent {
                std::iter::repeat(' ')
                    .take(self.indent_size * depth)
                    .collect::<String>()
            } else {
                std::iter::repeat(' ')
                    .take((self.indent_size * self.max_indent) - 3)
                    .collect::<String>()
                    + "-> "
            }
        } else {
            String::new()
        }
    }
}
