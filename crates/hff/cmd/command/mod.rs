use super::Result;
use clap::Subcommand;

mod dump;
pub use dump::*;

mod pack;
pub use pack::*;

mod unpack;
pub use unpack::*;

mod structure;
pub use structure::*;

/// Commands supported.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// The dump command.
    Dump(#[command(subcommand)] Dump),
    /// The pack command.
    Pack(#[command(subcommand)] Pack),
    /// The unpack command.
    Unpack(#[command(subcommand)] Unpack),
}

impl Commands {
    /// Execute the subcommand.
    pub fn execute(self) -> Result<()> {
        match self {
            Self::Dump(dump) => dump.execute(),
            Self::Pack(pack) => pack.execute(),
            Self::Unpack(unpack) => unpack.execute(),
        }
    }
}
