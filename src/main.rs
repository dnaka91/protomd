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

use anyhow::{Context, Result, bail};
use indexmap::IndexSet;
use itertools::Itertools;
use log::warn;
use protox::{
    Compiler,
    file::{ChainFileResolver, FileMetadata, IncludeFileResolver},
    prost_reflect::FileDescriptor,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use schemars::schema_for;
use walkdir::WalkDir;

use self::{
    cli::{Cli, Command},
    config::Conf,
    resolver::CachingFileResolver,
    templates::Package,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(cmd) = cli.cmd {
        match cmd {
            Command::Init => init()?,
            Command::Schema => schema()?,
            Command::Completion { dir } => cli::completion(&dir)?,
            Command::Manpages { dir } => cli::manpages(&dir)?,
        }

        return Ok(());
    }

    let config = config::load()?;

    let packages = collect(cli.include, cli.input, &config)?;
    render(cli.clean, &cli.output_dir, packages)?;

    Ok(())
}

fn collect(include: Vec<PathBuf>, input: Vec<PathBuf>, config: &Conf) -> Result<Vec<Package>> {
    let resolver = build_resolver(include);
    let files = search_inputs(input)?;

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

    Ok(templates)
}

fn render(clean: bool, output_dir: &Path, templates: Vec<Package>) -> Result<()> {
    if clean {
        clean_output(output_dir)?;
    }

    fs::create_dir_all(output_dir).ok();

    let env = templates::Env::new()?;

    templates.into_par_iter().try_for_each(|template| {
        let output_name = template.file_name();
        let file = File::create(output_dir.join(output_name))?;
        let mut file = BufWriter::with_capacity(256 * 1024, file);

        env.render(template, &mut file)?;
        file.flush()?;

        anyhow::Ok(())
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

fn init() -> Result<()> {
    if std::fs::exists("protomd.toml")? {
        bail!("A configuration file `protomd.toml` already exists");
    }

    std::fs::write("protomd.toml", config::template())?;
    Ok(())
}

fn schema() -> Result<()> {
    let schema = schema_for!(Package);
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}
