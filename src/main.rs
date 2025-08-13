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
use snafu::{ResultExt, Snafu, whatever};
use walkdir::WalkDir;

use self::{
    cli::{Cli, Command},
    config::Config,
    resolver::CachingFileResolver,
    templates::Package,
};

type Result<T, E = snafu::Whatever> = std::result::Result<T, E>;

#[snafu::report]
fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(cmd) = cli.cmd {
        match cmd {
            Command::Init => init()?,
            Command::Templates { dir, force } => templates(&dir, force)?,
            Command::Schema => schema()?,
            Command::Completion { dir } => {
                cli::completion(&dir).whatever_context("failed writing shell completions")?;
            }
            Command::Manpages { dir } => {
                cli::manpages(&dir).whatever_context("failed writing man pages")?;
            }
        }

        return Ok(());
    }

    let config = config::load().whatever_context("failed loading configuration")?;

    let packages = collect(cli.include, cli.input, &config)?;
    render(
        cli.clean,
        &cli.output_dir,
        packages,
        config.templates.as_deref(),
    )?;

    Ok(())
}

fn collect(include: Vec<PathBuf>, input: Vec<PathBuf>, config: &Config) -> Result<Vec<Package>> {
    let resolver = build_resolver(include);
    let files = search_inputs(input)?;

    let compiler = {
        let mut c = Compiler::with_file_resolver(resolver.clone());
        c.include_imports(true);
        c.include_source_info(true);
        c.open_files(files)
            .whatever_context("failed opening Protobuf files")?;
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

#[derive(Debug, Snafu)]
enum RenderError {
    #[snafu(display("failed creating output file at {path:?}"))]
    Create {
        source: std::io::Error,
        path: PathBuf,
    },
    Render {
        source: templates::RenderError,
    },
    Flush {
        source: std::io::Error,
    },
}

fn render(
    clean: bool,
    output_dir: &Path,
    templates: Vec<Package>,
    template_dir: Option<&str>,
) -> Result<()> {
    if clean {
        clean_output(output_dir)?;
    }

    fs::create_dir_all(output_dir).ok();

    let env = templates::Env::new(template_dir)?;

    templates
        .into_par_iter()
        .try_for_each(|template| {
            let path = output_dir.join(template.file_name());
            let file = File::create(&path).context(CreateSnafu { path })?;
            let mut file = BufWriter::with_capacity(256 * 1024, file);

            env.render(template, &mut file).context(RenderSnafu)?;
            file.flush().context(FlushSnafu)?;

            Ok::<_, RenderError>(())
        })
        .whatever_context("failed rendering files")?;

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
        let file_type = input
            .metadata()
            .whatever_context("failed reading file metadata")?
            .file_type();

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
                        .whatever_context("invalid entry")
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
    let current_dir = env::current_dir().whatever_context("failed finding current directory")?;
    let path = path
        .canonicalize()
        .whatever_context("failed canonicalizing output directory")?;

    if current_dir == path
        && fs::read_dir(&path)
            .whatever_context("failed listing directory")?
            .count()
            > 0
    {
        warn!("won't clean output directory as it points to the current directory");
        return Ok(());
    }

    fs::remove_dir_all(path).whatever_context("failed cleaning output directory")
}

fn should_generate(metadata: &HashMap<&str, &FileMetadata>, file: &FileDescriptor) -> bool {
    metadata.get(file.name()).is_some_and(|m| !m.is_import()) && file.services().count() > 0
}

fn init() -> Result<()> {
    if fs::exists("protomd.toml").whatever_context("failed checking file existence")? {
        whatever!("A configuration file `protomd.toml` already exists");
    }

    fs::write("protomd.toml", config::template())
        .whatever_context("failed writing configuration file")?;
    Ok(())
}

fn templates(dir: &Path, force: bool) -> Result<()> {
    if !force
        && fs::exists(dir).whatever_context("failed checking dir existence")?
        && fs::read_dir(dir)
            .whatever_context("failed listing directory")?
            .count()
            != 0
    {
        whatever!("The template directory is not empty");
    }

    fs::create_dir_all(dir).whatever_context("failed creating output directory")?;
    fs::write(
        dir.join("package.md.j2"),
        include_str!("../templates/package.md.j2").as_bytes(),
    )
    .whatever_context("failed writing template file")?;

    Ok(())
}

fn schema() -> Result<()> {
    let schema = schema_for!(Package);
    println!(
        "{}",
        serde_json::to_string_pretty(&schema).whatever_context("failed serializing schema")?
    );
    Ok(())
}
