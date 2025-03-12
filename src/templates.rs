use color_eyre::eyre::{ContextCompat, Result};
use itertools::Itertools;
use protox::{
    file::FileResolver,
    prost_reflect::{FileDescriptor, MessageDescriptor, MethodDescriptor, ServiceDescriptor},
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
    input: Message,
    output: Message,
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
            input: Message::new(resolver, &value.input())?,
            output: Message::new(resolver, &value.output())?,
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
    fn new(resolver: &impl FileResolver, value: &MessageDescriptor) -> Result<Self> {
        let source = value.parent_file();
        let file = resolver.open_file(source.name())?;

        let source = source
            .file_descriptor_proto()
            .source_code_info
            .as_ref()
            .context("missing source info")?;

        let location = source
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
