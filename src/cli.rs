use std::{
    fs, io,
    path::{Path, PathBuf},
};

use clap::{CommandFactory, Parser, Subcommand, ValueEnum, ValueHint};
use clap_complete::Shell;
use clap_complete_nushell::Nushell;

/// Markdow generator for Protobuf schema files.
#[derive(Parser)]
pub struct Cli {
    /// Directories that will be searched for referenced schema files.
    ///
    /// Input files can reference other schemas that might not be part of the total list of input
    /// files. Each include path given will be tried when locating the include files.
    #[arg(short = 'I', long, value_hint = ValueHint::DirPath)]
    pub include: Vec<PathBuf>,

    /// Directory to write the generated files into.
    ///
    /// If the directory doesn't exist, it will be created automatically.
    ///
    /// Any existing files will be overwritten without warning. Therefore, be careful if pointing
    /// to a directory that already includes other files. Files that don't have any overlapping
    /// name remain untouched.
    ///
    /// Use the `--clean` flag to wipe the output directory before generating the new files.
    #[arg(short, long, value_hint = ValueHint::DirPath, default_value_os_t = PathBuf::from("."))]
    pub output_dir: PathBuf,

    /// Remove any content from the output directory before writing any files to it. This is not
    /// done when the output directory points to the current directory.
    #[arg(long)]
    pub clean: bool,

    /// Input files or folders to generate the documentation from.
    ///
    /// In case of a file, it is only included if it has a `*.proto` extension. However, if pointed
    /// to a directory, it will be searched recursively for `*.proto` files.
    pub input: Vec<PathBuf>,

    #[command(subcommand)]
    pub cmd: Option<Command>,
}

impl Cli {
    #[allow(dead_code)]
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new configuration file under the current working directory.
    ///
    /// If the file already exists, it will not be overwritten but an error be printed out instead.
    /// To overwrite an existing configuration, delete it first and then run the command again.
    ///
    /// Using this flag will ignore all other arguments, only generate the initial configuration
    /// file and then exit.
    Init,

    /// Print the schema of the template context on STDOUT.
    ///
    /// This documents the structure of the data that is provided to the Jinja template when it is
    /// rendered and can help when building a custom template to be used instead of the default.
    Schema,

    /// Create shell completion scripts for all supported shells.
    ///
    /// The completions will be written in the given directory, with an appropriate naming for each
    /// shell type. For example, *.bash, *.elv, *.fish, ...
    Completion {
        /// Directory to place create the files in. If the directory doesn't exist already, it'll be
        /// created.
        ///
        /// Note that any existing file will be overwritten without further confirmation or warning.
        #[arg(value_hint = ValueHint::DirPath)]
        dir: PathBuf,
    },

    /// Create `man` page files with documentation about all options and subcommands.
    Manpages {
        /// Directory to place create the files in. If the directory doesn't exist already, it'll be
        /// created.
        ///
        /// Note that any existing file will be overwritten without further confirmation or warning.
        #[arg(value_hint = ValueHint::DirPath)]
        dir: PathBuf,
    },
}

pub fn completion(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir).ok();

    let mut cmd = Cli::command();
    let bin_name = cmd.get_bin_name().unwrap_or(cmd.get_name()).to_owned();

    for &shell in Shell::value_variants() {
        clap_complete::generate_to(shell, &mut cmd, &bin_name, dir)?;
    }

    clap_complete::generate_to(Nushell, &mut cmd, &bin_name, dir)?;

    Ok(())
}

pub fn manpages(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir).ok();

    let cmd = Cli::command();
    clap_mangen::generate_to(cmd, dir)
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
