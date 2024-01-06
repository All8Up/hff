use clap::Parser;
use std::path::PathBuf;

/// Options for the dump command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
pub struct Options {
    /// Source directory to package.
    #[arg(long, short)]
    pub input: PathBuf,
    /// If the scan should be non-recursive.
    #[arg(long, short)]
    pub non_recursive: bool,
    /// Compression level for the content?
    #[arg(long, short)]
    pub compression: Option<u32>,

    /// Endian mode.
    #[arg(long, short)]
    pub big_endian: bool,

    /// File to create.
    #[arg(long, short)]
    pub output: PathBuf,
}
