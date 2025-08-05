module completions {

  # Markdow generator for Protobuf schema files
  export extern protomd [
    --include(-I): path       # Directories that will be searched for referenced schema files
    --output-dir(-o): path    # Directory to write the generated files into
    --clean                   # Remove any content from the output directory before writing any files to it. This is not done when the output directory points to the current directory
    ...input: path            # Input files or folders to generate the documentation from
    --help(-h)                # Print help (see more with '--help')
  ]

  # Initialize a new configuration file under the current working directory
  export extern "protomd init" [
    --help(-h)                # Print help (see more with '--help')
  ]

  export extern "protomd templates" [
    --force(-f)               # Force creating files if the target directory isn't empty
    dir?: path                # Directory to create the files in. If the directory doesn't exist already, it'll be created
    --help(-h)                # Print help (see more with '--help')
  ]

  # Print the schema of the template context on STDOUT
  export extern "protomd schema" [
    --help(-h)                # Print help (see more with '--help')
  ]

  # Create shell completion scripts for all supported shells
  export extern "protomd completion" [
    dir: path                 # Directory to create the files in. If the directory doesn't exist already, it'll be created
    --help(-h)                # Print help (see more with '--help')
  ]

  # Create `man` page files with documentation about all options and subcommands
  export extern "protomd manpages" [
    dir: path                 # Directory to create the files in. If the directory doesn't exist already, it'll be created
    --help(-h)                # Print help (see more with '--help')
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "protomd help" [
  ]

  # Initialize a new configuration file under the current working directory
  export extern "protomd help init" [
  ]

  export extern "protomd help templates" [
  ]

  # Print the schema of the template context on STDOUT
  export extern "protomd help schema" [
  ]

  # Create shell completion scripts for all supported shells
  export extern "protomd help completion" [
  ]

  # Create `man` page files with documentation about all options and subcommands
  export extern "protomd help manpages" [
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "protomd help help" [
  ]

}

export use completions *
