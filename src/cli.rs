use std::path::PathBuf;

use clap::{Parser, ValueHint};

/// Markdow generator for Protobuf schema files.
#[derive(Parser)]
pub struct Cli {
    #[arg(short = 'I', long, value_hint = ValueHint::DirPath)]
    pub include: Vec<PathBuf>,
    /// Directory to write the generated files into.
    ///
    /// If the directory doesn't exist, it will be created automatically.
    ///
    /// Any existing files will be overwritten without warning. Therefore, be careful if pointing to
    /// a directory that already includes other files. Files that don't have any overlapping name
    /// remain untouched.
    ///
    /// Use the `--clean` flag to wipe the output directory before generating the new files.
    #[arg(short, long, value_hint = ValueHint::DirPath, default_value_os_t = PathBuf::from("."))]
    pub output_dir: PathBuf,
    /// Remove any content from the output directory before writing any files to it. This is not
    /// done when the output directory points to the current directory.
    #[arg(long)]
    pub clean: bool,
    /// Initialize a new configuration file under the current working directory.
    #[arg(long)]
    pub init: bool,
    /// Input files or folders to generate the documentation from.
    ///
    /// In case of a file, it is only included if it has a `*.proto` extension. However, if pointed
    /// to a directory, it will be searched recursively for `*.proto` files.
    pub input: Vec<PathBuf>,
}

impl Cli {
    #[allow(dead_code)]
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
#[cfg(test)]
mod tests {
    use crate::cli::Cli;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
