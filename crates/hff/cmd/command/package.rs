use super::Result;
use clap::Args;
use std::path::PathBuf;

/// Package a file or directory into an hff container.
#[derive(Debug, Args)]
pub struct Package {
    /// The input path to a file or directory.
    pub input: PathBuf,
}

impl Package {
    /// Execute the subcommand.
    pub fn execute(self) -> Result<()> {
        Ok(())
    }
}
