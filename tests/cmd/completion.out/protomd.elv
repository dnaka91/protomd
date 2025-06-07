
use builtin;
use str;

set edit:completion:arg-completer[protomd] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'protomd'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'protomd'= {
            cand -I 'Directories that will be searched for referenced schema files'
            cand --include 'Directories that will be searched for referenced schema files'
            cand -o 'Directory to write the generated files into'
            cand --output-dir 'Directory to write the generated files into'
            cand --clean 'Remove any content from the output directory before writing any files to it. This is not done when the output directory points to the current directory'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand init 'Initialize a new configuration file under the current working directory'
            cand schema 'Print the schema of the template context on STDOUT'
            cand completion 'Create shell completion scripts for all supported shells'
            cand manpages 'Create `man` page files with documentation about all options and subcommands'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'protomd;init'= {
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'protomd;schema'= {
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'protomd;completion'= {
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'protomd;manpages'= {
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'protomd;help'= {
            cand init 'Initialize a new configuration file under the current working directory'
            cand schema 'Print the schema of the template context on STDOUT'
            cand completion 'Create shell completion scripts for all supported shells'
            cand manpages 'Create `man` page files with documentation about all options and subcommands'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'protomd;help;init'= {
        }
        &'protomd;help;schema'= {
        }
        &'protomd;help;completion'= {
        }
        &'protomd;help;manpages'= {
        }
        &'protomd;help;help'= {
        }
    ]
    $completions[$command]
}
