use clap::Parser;

/// Options for the dump command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
pub struct Options {
    /// Name of the file to dump.
    #[arg()]
    pub name: std::path::PathBuf,
    /// Indent the information by depth?
    #[arg(long)]
    pub no_indent: bool,
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
    #[arg(long, default_value = "false")]
    pub as_ksv: bool,
    /// Interpret the metadata as a string vector.
    #[arg(long)]
    pub as_string_vector: bool,
}
