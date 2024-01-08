use super::Result;
use clap::Args;
use std::path::PathBuf;

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
        Ok(())
    }
}
