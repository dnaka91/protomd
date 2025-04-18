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
    /// Symbols used to define the gRPC method type in templates.
    ///
    /// Note that these are always encapsulated in backticks to better highlight them in the
    /// resulting Markdown, therefore HTML.
    #[config(nested)]
    pub request_symbols: RequestSymbols,
}

#[derive(Clone, confique::Config, JsonSchema, Serialize)]
pub struct Markdownlint {
    /// List of rules to disable.
    #[config(default = [])]
    pub disable: Vec<String>,
}

/// Symbols, as in any kind of textual identifier, to describe the different kinds of gRPC calls.
#[derive(Clone, confique::Config, JsonSchema, Serialize)]
pub struct RequestSymbols {
    #[config(default = "unary")]
    pub unary: String,
    /// Client-side streaming.
    #[config(default = "client streaming")]
    pub client_streaming: String,
    /// Server-side streaming.
    #[config(default = "server streaming")]
    pub server_streaming: String,
    /// Bidirectional (both client- and server-side) streaming.
    #[config(default = "bidirectional streaming")]
    pub bidi_streaming: String,
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
