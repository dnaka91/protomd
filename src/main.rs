#![warn(
    rust_2018_idioms,
    clippy::all,
    clippy::pedantic,
    clippy::expect_used,
    clippy::unwrap_used
)]
#![allow(unstable_name_collisions, clippy::cast_sign_loss)]

mod cli;
mod config;
mod resolver;
mod templates;

use std::{
    collections::HashMap,
    env,
    fs::{self, File, FileType},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use color_eyre::{
    Section,
    eyre::{self, Context, Result, eyre},
};
use indexmap::IndexSet;
use itertools::Itertools;
use log::warn;
use protox::{
    Compiler,
    file::{ChainFileResolver, FileMetadata, IncludeFileResolver},
    prost_reflect::FileDescriptor,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rinja::Template;
use walkdir::WalkDir;

use self::{cli::Cli, resolver::CachingFileResolver, templates::Package};

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    if cli.init {
        if std::fs::exists("protomd.toml")? {
            return Err(eyre!("A configuration file `protomd.toml` already exists")
                .suggestion("Consider deleting the existing file and run the command again"));
        }

        std::fs::write("protomd.toml", config::template())?;
        return Ok(());
    }

    let config = config::load()?;

    let resolver = build_resolver(cli.include);
    let files = search_inputs(cli.input)?;

    let compiler = {
        let mut c = Compiler::with_file_resolver(resolver.clone());
        c.include_imports(true);
        c.include_source_info(true);
        c.open_files(files)?;
        c
    };

    let metadata = compiler
        .files()
        .map(|f| (f.name(), f))
        .collect::<HashMap<_, _>>();

    let packages = compiler
        .descriptor_pool()
        .files()
        .filter(|file| should_generate(&metadata, file))
        .into_group_map_by(|file| file.package_name().to_owned());

    let templates = packages
        .into_iter()
        .map(|(name, files)| Package::new(config.clone(), &resolver, name, &files))
        .collect::<Result<Vec<_>, _>>()?;

    if cli.clean {
        clean_output(&cli.output_dir)?;
    }

    fs::create_dir_all(&cli.output_dir).ok();

    templates.into_par_iter().try_for_each(|template| {
        let output_name = template.file_name();
        let file = File::create(cli.output_dir.join(output_name))?;
        let mut file = BufWriter::with_capacity(256 * 1024, file);

        template.write_into(&mut file)?;
        file.flush()?;

        eyre::Ok(())
    })?;

    Ok(())
}

fn build_resolver(includes: Vec<PathBuf>) -> CachingFileResolver<ChainFileResolver> {
    let mut chain = ChainFileResolver::new();
    for include in includes {
        chain.add(IncludeFileResolver::new(include));
    }

    CachingFileResolver::new(chain)
}

fn search_inputs(inputs: Vec<PathBuf>) -> Result<IndexSet<PathBuf>> {
    let mut files = IndexSet::new();

    for input in inputs {
        let file_type = input.metadata()?.file_type();

        if file_type.is_dir() {
            let resolved = WalkDir::new(input)
                .min_depth(1)
                .into_iter()
                .filter_map(|entry| {
                    entry
                        .map(|entry| {
                            if is_proto(entry.file_type(), entry.path()) {
                                Some(entry.into_path())
                            } else {
                                None
                            }
                        })
                        .map_err(Into::into)
                        .transpose()
                })
                .collect::<Result<Vec<_>>>()?;

            files.extend(resolved);
        } else if is_proto(file_type, &input) {
            files.insert(input);
        }
    }
    Ok(files)
}

fn is_proto(file_type: FileType, path: impl AsRef<Path>) -> bool {
    file_type.is_file()
        && path
            .as_ref()
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("proto"))
}

fn clean_output(path: &Path) -> Result<()> {
    let current_dir = env::current_dir().context("failed finding current directory")?;
    let path = path
        .canonicalize()
        .context("failed canonicalizing output directory")?;

    if current_dir == path && fs::read_dir(&path)?.count() > 0 {
        warn!("won't clean output directory as it points to the current directory");
        return Ok(());
    }

    fs::remove_dir_all(path).context("failed cleaning output directory")
}

fn should_generate(metadata: &HashMap<&str, &FileMetadata>, file: &FileDescriptor) -> bool {
    metadata.get(file.name()).is_some_and(|m| !m.is_import()) && file.services().count() > 0
}
