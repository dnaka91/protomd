# Protobuf Markdown Renderer

Simplistic and standalone Protobuf schema to Markdown renderer, with a focus on nice output for Vitepress and gRPC services.

- Fully standalone, no need to have the `protoc` compiler installed.
- _Clean_ Markdown output that passes all default `markdownlint` rules. [^1]

[^1]: For the overall output. Documentation comments in schema files are not taken into account and transferred to Markdown as is.

## Naming

**PROTO**buf **M**ark**D**own → protomd

## Usage

First of all, some Protobuf schema files are required as input files, as with no input data there's nothing to process and no output will be created. The current focus is on documenting the `service`s and their `rpc`s. Thus, a plain schema file with only `message`s and `enum`s won't create any output either.

For demo purposes let's assume this simple schema, saved as `sample.proto`:

```proto
syntax = "proto3";

package markdown.sample;

// This is a simple message.
message Simple {
  // A single integer.
  uint32 value = 1;
}

// The simplest server.
service SimpleService {
  // Call it!
  rpc Call(Simple) returns (Simple);
}
```

We can run the following command to generate a Markdown file for it:

```sh
protomd -I . sample.proto
```

This might look familiar to those used to invoking `protoc` directly and that is no mistake. The interface style of the Protobuf compiler is copied to keep it simple and familiar.

`-I .` describes an import path which points to the current directory. No much of relevance for a single file, but in case we reference additional schemas, this helps to locate said files.

The output file will be `markdown.sample.md` in this case. The file name is always the package name (`markdown.sample`) plus the `.md` file extension.

## License

This project is licensed under [MIT License](LICENSE.md) (or <http://opensource.org/licenses/MIT>).
