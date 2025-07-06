use anyhow::Result;
use config::{Case, Environment, File};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuration for the protomd Protobuf Markdown generator.
#[derive(Clone, Default, Deserialize, JsonSchema, Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Config {
    /// Optional frontmatter to place at the beginning of each generated file.
    ///
    /// For example, in Vitepress you can configure the outline level for the navigation sidebar by
    /// adding the following configuration as frontmatter:
    ///
    /// frontmatter = "outline: [2, 4]"
    #[serde(default)]
    pub frontmatter: String,
    /// Configuration for `markdownlint`.
    #[serde(default)]
    pub markdownlint: Markdownlint,
    /// Symbols used to define the gRPC method type in templates.
    #[serde(default)]
    pub request_symbols: RequestSymbols,
}

/// Configuration for `markdownlint`.
#[derive(Clone, Default, Deserialize, JsonSchema, Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Markdownlint {
    /// List of rules to disable.
    #[serde(default)]
    pub disable: Vec<String>,
}

/// Symbols used to define the gRPC method type in templates.
///
/// Note that these are always encapsulated in backticks to better highlight them in the
/// resulting Markdown, therefore HTML.
#[derive(Clone, Default, Deserialize, JsonSchema, Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct RequestSymbols {
    /// Single request-response oneshot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unary: Option<String>,
    /// Client-side streaming.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_streaming: Option<String>,
    /// Server-side streaming.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_streaming: Option<String>,
    /// Bidirectional (both client- and server-side) streaming.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bidi_streaming: Option<String>,
}

pub fn load() -> Result<Config> {
    config::Config::builder()
        .add_source(File::with_name(".config/protomd.toml").required(false))
        .add_source(File::with_name("protomd.toml").required(false))
        .add_source(Environment::with_prefix("PROTOMD_").convert_case(Case::ScreamingSnake))
        .build()?
        .try_deserialize()
        .map_err(Into::into)
}

pub fn template() -> &'static str {
    include_str!("config.toml")
}

#[cfg(test)]
mod tests {
    use config::FileFormat;

    use super::*;

    #[test]
    fn deser_default() -> Result<()> {
        let cfg = config::Config::builder()
            .add_source(File::from_str(template(), FileFormat::Toml))
            .build()?
            .try_deserialize::<Config>()?;

        assert_eq!(Config::default(), cfg);

        Ok(())
    }
}
