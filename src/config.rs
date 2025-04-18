use anyhow::Result;
use confique::{Config, toml::FormatOptions};
use schemars::JsonSchema;
use serde::Serialize;

/// Configuration for the protomd Protobuf Markdown generator.
#[derive(Clone, confique::Config, JsonSchema, Serialize)]
pub struct Conf {
    /// Optional frontmatter to place at the beginning of each generated file.
    ///
    /// For example, in Vitepress you can configure the outline level for the navigation sidebar by
    /// adding the following configuration as frontmatter:
    ///
    /// frontmatter = "outline: [2, 4]"
    #[config(default = "")]
    pub frontmatter: String,
    /// Configuration for `markdownlint`.
    #[config(nested)]
    pub markdownlint: Markdownlint,
}

#[derive(Clone, confique::Config, JsonSchema, Serialize)]
pub struct Markdownlint {
    /// List of rules to disable.
    #[config(default = [])]
    pub disable: Vec<String>,
}

pub fn load() -> Result<Conf> {
    Conf::builder()
        .file(".config/protomd.toml")
        .file("protomd.toml")
        .load()
        .map_err(Into::into)
}

pub fn template() -> String {
    confique::toml::template::<Conf>(FormatOptions::default())
}
