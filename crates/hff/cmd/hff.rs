//! A command line tool to work with hff containers.
#![warn(missing_docs)]

use clap::Parser;
use hff_std::*;
use log::info;

mod args;
use args::*;

mod command;
pub use command::*;

fn main() -> Result<()> {
    // Parse the command line options.
    let args = Args::parse();

    // Setup logging.
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    // Log the options.
    info!("{:#?}", args);

    // Call the subcommand.
    args.command.execute()?;

    Ok(())
}
