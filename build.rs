use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use clap::{CommandFactory, ValueEnum};
use clap_complete::Shell;

mod cli {
    include!("src/cli.rs");
}

#[allow(clippy::unwrap_used)]
fn main() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    completion(&manifest_dir)?;
    manpages(&manifest_dir)?;

    Ok(())
}

fn completion(base_dir: &Path) -> io::Result<()> {
    let out_dir = base_dir.join("completion");

    fs::create_dir_all(&out_dir).ok();

    let mut cmd = cli::Cli::command();
    for &shell in Shell::value_variants() {
        clap_complete::generate_to(shell, &mut cmd, "myapp", &out_dir)?;
    }

    Ok(())
}

fn manpages(base_dir: &Path) -> io::Result<()> {
    let out_dir = base_dir.join("manpages");

    fs::create_dir_all(&out_dir).ok();

    let cmd = cli::Cli::command();
    clap_mangen::generate_to(cmd, out_dir)?;

    Ok(())
}
