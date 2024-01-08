use super::Result;
use clap::Subcommand;

mod dump;
pub use dump::*;

mod package;
pub use package::*;

mod unpack;
pub use unpack::*;

/// Commands supported.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// The dump command.
    Dump(#[command(subcommand)] Dump),
    /// The package command.
    Package(#[command(subcommand)] Package),
    /// The unpack command.
    Unpack(#[command(subcommand)] Unpack),
}

impl Commands {
    /// Execute the subcommand.
    pub fn execute(self) -> Result<()> {
        match self {
            Self::Dump(dump) => dump.execute(),
            Self::Package(package) => package.execute(),
            Self::Unpack(unpack) => unpack.execute(),
        }
    }
}
