use anyhow::{Context, Result};
use indexmap::IndexMap;
use itertools::Itertools;
use protox::{
    file::FileResolver,
    prost_reflect::{
        EnumDescriptor, FieldDescriptor, FileDescriptor, Kind, MessageDescriptor, MethodDescriptor,
        ServiceDescriptor,
    },
};
use rinja::Template;

use crate::config;

#[derive(Template)]
#[template(path = "package.md.j2")]
pub struct Package {
    config: config::Conf,
    name: String,
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

#[derive(Template)]
#[template(path = "service.md.j2")]
struct Service {
    name: String,
    description: String,
    methods: Vec<Method>,
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

        Ok(Self {
            name: value.name().to_owned(),
            description,
            methods: value
                .methods()
                .map(|v| Method::new(resolver, &v))
                .collect::<Result<_>>()?,
        })
    }
}

#[derive(Template)]
#[template(path = "method.md.j2")]
struct Method {
    name: String,
    description: String,
    input: IndexMap<String, Message>,
    output: IndexMap<String, Message>,
    client_streaming: bool,
    server_streaming: bool,
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

        Ok(Self {
            name: value.name().to_owned(),
            description,
            input: find_messages(resolver, value.input())?,
            output: find_messages(resolver, value.output())?,
            client_streaming: value.is_client_streaming(),
            server_streaming: value.is_server_streaming(),
        })
    }
}

#[derive(Template)]
#[template(path = "message.md.j2")]
struct Message {
    description: String,
    proto: String,
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

        Ok(Self { description, proto })
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

enum CombinedDescriptor {
    Message(MessageDescriptor),
    Enum(EnumDescriptor),
}

impl CombinedDescriptor {
    fn parent_file(&self) -> FileDescriptor {
        match self {
            Self::Message(d) => d.parent_file(),
            Self::Enum(d) => d.parent_file(),
        }
    }

    fn full_name(&self) -> &str {
        match self {
            Self::Message(d) => d.full_name(),
            Self::Enum(d) => d.full_name(),
        }
    }

    fn path(&self) -> &[i32] {
        match self {
            Self::Message(d) => d.path(),
            Self::Enum(d) => d.path(),
        }
    }

    fn is_map_entry(&self) -> bool {
        match self {
            Self::Message(d) => d.is_map_entry(),
            Self::Enum(_) => false,
        }
    }

    fn fields(&self) -> Box<dyn ExactSizeIterator<Item = FieldDescriptor> + '_> {
        match self {
            Self::Message(d) => Box::new(d.fields()),
            Self::Enum(_) => Box::new(std::iter::empty()),
        }
    }

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
