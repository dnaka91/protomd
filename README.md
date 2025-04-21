# Protobuf Markdown Renderer

Simplistic and standalone Protobuf schema to Markdown renderer, with a focus on nice output for Vitepress and gRPC services.

- Fully standalone, no need to have the `protoc` compiler installed.
- _Clean_ Markdown output that passes all default `markdownlint` rules. [^1]

[^1]: For the overall output. Documentation comments in schema files is not taken into account and transferred to Markdown as is.

## Naming

**PROTO**buf **M**ark**D**own → protomd

## License

This project is licensed under [MIT License](LICENSE.md) (or <http://opensource.org/licenses/MIT>).
