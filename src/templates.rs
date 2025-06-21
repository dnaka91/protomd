use std::io::Write;

use anyhow::{Context, Result};
use indexmap::IndexMap;
use itertools::Itertools;
use minijinja::Environment;
use protox::{
    file::FileResolver,
    prost_reflect::{
        EnumDescriptor, FieldDescriptor, FileDescriptor, Kind, MessageDescriptor, MethodDescriptor,
        ServiceDescriptor,
    },
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::config;

mod filters {
    pub fn slugify(s: String) -> String {
        slug::slugify(s)
    }
}

pub struct Env(Environment<'static>);

impl Env {
    pub fn new() -> Result<Self> {
        let mut env = Environment::new();
        env.add_filter("slugify", filters::slugify);
        env.add_template("package.md.j2", include_str!("../templates/package.md.j2"))?;

        Ok(Self(env))
    }

    pub fn render(&self, package: Package, writer: impl Write) -> Result<()> {
        self.0
            .get_template("package.md.j2")?
            .render_to_write(package, writer)
            .map(|_| ())
            .map_err(Into::into)
    }
}

/// A Protobuf package which contains services and methods. Maybe originate from multiple schema
/// files.
#[derive(JsonSchema, Serialize)]
pub struct Package {
    /// Configuration of `protomd` and not part of the schema definition.
    config: config::Conf,
    /// The package name.
    name: String,
    /// List of services defined in the package.
    services: Vec<Service>,
}

impl Package {
    pub fn new(
        config: config::Conf,
        resolver: &impl FileResolver,
        name: String,
        value: &[FileDescriptor],
    ) -> Result<Self> {
        Ok(Self {
            config,
            name,
            services: value
                .iter()
                .flat_map(FileDescriptor::services)
                .map(|v| Service::new(resolver, &v))
                .collect::<Result<_>>()?,
        })
    }

    pub fn file_name(&self) -> String {
        format!("{}.md", self.name)
    }
}

/// A gRPC service as part of a package.
#[derive(JsonSchema, Serialize)]
struct Service {
    /// Name of the gRPC service.
    name: String,
    /// Description of the gRPC service.
    description: String,
    /// List of methods the service provides.
    methods: Vec<Method>,
    /// Whether this service is marked deprecated.
    deprecated: bool,
    /// Whether the file this service is defined in is marked deprecated.
    file_deprecated: bool,
}

impl Service {
    fn new(resolver: &impl FileResolver, value: &ServiceDescriptor) -> Result<Self> {
        let source = value
            .parent_file_descriptor_proto()
            .source_code_info
            .as_ref()
            .context("missing source info")?;

        let description = source
            .location
            .iter()
            .find(|l| l.path == value.path())
            .map(|l| unindent::unindent(l.leading_comments().trim()))
            .unwrap_or_default();

        let deprecated = value
            .service_descriptor_proto()
            .options
            .as_ref()
            .and_then(|o| o.deprecated)
            .unwrap_or(false);

        let file_deprecated = value
            .parent_file_descriptor_proto()
            .options
            .as_ref()
            .and_then(|o| o.deprecated)
            .unwrap_or(false);

        Ok(Self {
            name: value.name().to_owned(),
            description,
            methods: value
                .methods()
                .map(|v| Method::new(resolver, &v))
                .collect::<Result<_>>()?,
            deprecated,
            file_deprecated,
        })
    }
}

/// A gRPC method as part of a service.
#[derive(JsonSchema, Serialize)]
struct Method {
    /// Name of the method.
    name: String,
    /// Description of the method.
    description: String,
    /// Input method parameter to the method call.
    input: IndexMap<String, Message>,
    /// Output message parameter to the method call.
    output: IndexMap<String, Message>,
    /// Whether this method uses client-side streaming.
    client_streaming: bool,
    /// Whether this method uses server-side streaming.
    server_streaming: bool,
    /// Whether this method is marked deprecated.
    deprecated: bool,
}

impl Method {
    fn new(resolver: &impl FileResolver, value: &MethodDescriptor) -> Result<Self> {
        let source = value.parent_file();
        let source = source
            .file_descriptor_proto()
            .source_code_info
            .as_ref()
            .context("missing source info")?;

        let description = source
            .location
            .iter()
            .find(|l| l.path == value.path())
            .map(|l| unindent::unindent(l.leading_comments().trim()))
            .unwrap_or_default();

        let deprecated = value
            .method_descriptor_proto()
            .options
            .as_ref()
            .and_then(|o| o.deprecated)
            .unwrap_or(false);

        Ok(Self {
            name: value.name().to_owned(),
            description,
            input: find_messages(resolver, value.input())?,
            output: find_messages(resolver, value.output())?,
            client_streaming: value.is_client_streaming(),
            server_streaming: value.is_server_streaming(),
            deprecated,
        })
    }
}

/// A Protobuf message, referenced by a method as parameter or nested within.
#[derive(JsonSchema, Serialize)]
struct Message {
    /// Description of the message.
    description: String,
    /// Raw Protobuf schema definition.
    proto: String,
    /// Whether this message is marked deprecated.
    deprecated: bool,
}

impl Message {
    fn new(resolver: &impl FileResolver, value: &CombinedDescriptor) -> Result<Self> {
        let source = value.parent_file();
        let file = resolver.open_file(source.name())?;

        let source_info = source
            .file_descriptor_proto()
            .source_code_info
            .as_ref()
            .context("missing source info")?;

        let location = source_info
            .location
            .iter()
            .find(|l| l.path == value.path())
            .context("missing location for message")?;

        let description = unindent::unindent(location.leading_comments().trim());
        let proto = file
            .source()
            .map(|source| {
                let start = location.span[0] as usize;
                let end = if location.span.len() == 4 {
                    location.span[2]
                } else {
                    location.span[0]
                } as usize;

                source
                    .lines()
                    .skip(start)
                    .take(end - start + 1)
                    .intersperse("\n")
                    .collect::<String>()
            })
            .unwrap_or_default();
        let proto = unindent::unindent(&proto);
        let deprecated = value.deprecated();

        Ok(Self {
            description,
            proto,
            deprecated,
        })
    }
}

fn find_messages(
    resolver: &impl FileResolver,
    value: MessageDescriptor,
) -> Result<IndexMap<String, Message>> {
    let descriptor = CombinedDescriptor::Message(value);
    let mut messages = IndexMap::from_iter([(
        descriptor.full_name().to_owned(),
        Message::new(resolver, &descriptor)?,
    )]);

    collect_deps(resolver, &mut messages, &descriptor)?;

    Ok(messages)
}

fn collect_deps(
    resolver: &impl FileResolver,
    deps: &mut IndexMap<String, Message>,
    message: &CombinedDescriptor,
) -> Result<()> {
    for field in message.fields() {
        let field = match field.kind() {
            Kind::Message(m) if m.is_map_entry() => m.map_entry_value_field(),
            _ => field,
        };

        let descriptor = match field.kind() {
            Kind::Message(d) => CombinedDescriptor::from(d),
            Kind::Enum(d) => CombinedDescriptor::from(d),
            _ => continue,
        };

        assert!(!descriptor.is_map_entry());

        if descriptor.included_in(message) {
            continue;
        }

        deps.insert(
            descriptor.full_name().to_owned(),
            Message::new(resolver, &descriptor)?,
        );

        collect_deps(resolver, deps, &descriptor)?;
    }

    Ok(())
}

/// Combined logic for messages and enum, as most of the accessors and logic is the same regadless.
///
/// This allows to have much less match statements in the code that uses it as we can glue over all
/// the minor details inside of this type.
enum CombinedDescriptor {
    /// Protobuf message.
    Message(MessageDescriptor),
    /// Protobuf enum.
    Enum(EnumDescriptor),
}

impl CombinedDescriptor {
    /// Get the parent file this message/enum is defined in.
    fn parent_file(&self) -> FileDescriptor {
        match self {
            Self::Message(d) => d.parent_file(),
            Self::Enum(d) => d.parent_file(),
        }
    }

    /// Get the full name of this message/enum, which is the package name plus its own name.
    fn full_name(&self) -> &str {
        match self {
            Self::Message(d) => d.full_name(),
            Self::Enum(d) => d.full_name(),
        }
    }

    /// Get the path to locate the descriptor's source code.
    fn path(&self) -> &[i32] {
        match self {
            Self::Message(d) => d.path(),
            Self::Enum(d) => d.path(),
        }
    }

    /// Whether this is an auto-generated descriptor for map entries.
    fn is_map_entry(&self) -> bool {
        match self {
            Self::Message(d) => d.is_map_entry(),
            Self::Enum(_) => false,
        }
    }

    /// Iterate over al the fields in the descriptor.
    fn fields(&self) -> Box<dyn ExactSizeIterator<Item = FieldDescriptor> + '_> {
        match self {
            Self::Message(d) => Box::new(d.fields()),
            Self::Enum(_) => Box::new(std::iter::empty()),
        }
    }

    /// Whether this message/enum is included in the given other descriptor.
    fn included_in(&self, other: &CombinedDescriptor) -> bool {
        let parent = match self {
            Self::Message(d) => d.parent_message(),
            Self::Enum(d) => d.parent_message(),
        };

        let other = match other {
            Self::Message(d) => Some(d),
            Self::Enum(_) => None,
        };

        parent.as_ref() == other
    }

    /// Whether this message/enum is marked as deprecated.
    fn deprecated(&self) -> bool {
        match self {
            CombinedDescriptor::Message(d) => d
                .descriptor_proto()
                .options
                .as_ref()
                .and_then(|o| o.deprecated)
                .unwrap_or(false),
            CombinedDescriptor::Enum(d) => d
                .enum_descriptor_proto()
                .options
                .as_ref()
                .and_then(|o| o.deprecated)
                .unwrap_or(false),
        }
    }
}

impl From<MessageDescriptor> for CombinedDescriptor {
    fn from(value: MessageDescriptor) -> Self {
        Self::Message(value)
    }
}

impl From<EnumDescriptor> for CombinedDescriptor {
    fn from(value: EnumDescriptor) -> Self {
        Self::Enum(value)
    }
}
