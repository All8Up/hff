use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};

use super::Commands;

/// Command line options.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
pub struct Args {
    /// Verbosity level.
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    /// Subcommand.
    #[command(subcommand)]
    pub command: Commands,
}
