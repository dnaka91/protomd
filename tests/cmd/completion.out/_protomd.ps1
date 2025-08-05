
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'protomd' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'protomd'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'protomd' {
            [CompletionResult]::new('-I', '-I ', [CompletionResultType]::ParameterName, 'Directories that will be searched for referenced schema files')
            [CompletionResult]::new('--include', '--include', [CompletionResultType]::ParameterName, 'Directories that will be searched for referenced schema files')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Directory to write the generated files into')
            [CompletionResult]::new('--output-dir', '--output-dir', [CompletionResultType]::ParameterName, 'Directory to write the generated files into')
            [CompletionResult]::new('--clean', '--clean', [CompletionResultType]::ParameterName, 'Remove any content from the output directory before writing any files to it. This is not done when the output directory points to the current directory')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize a new configuration file under the current working directory')
            [CompletionResult]::new('templates', 'templates', [CompletionResultType]::ParameterValue, 'templates')
            [CompletionResult]::new('schema', 'schema', [CompletionResultType]::ParameterValue, 'Print the schema of the template context on STDOUT')
            [CompletionResult]::new('completion', 'completion', [CompletionResultType]::ParameterValue, 'Create shell completion scripts for all supported shells')
            [CompletionResult]::new('manpages', 'manpages', [CompletionResultType]::ParameterValue, 'Create `man` page files with documentation about all options and subcommands')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'protomd;init' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'protomd;templates' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Force creating files if the target directory isn''t empty')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'Force creating files if the target directory isn''t empty')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'protomd;schema' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'protomd;completion' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'protomd;manpages' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'protomd;help' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize a new configuration file under the current working directory')
            [CompletionResult]::new('templates', 'templates', [CompletionResultType]::ParameterValue, 'templates')
            [CompletionResult]::new('schema', 'schema', [CompletionResultType]::ParameterValue, 'Print the schema of the template context on STDOUT')
            [CompletionResult]::new('completion', 'completion', [CompletionResultType]::ParameterValue, 'Create shell completion scripts for all supported shells')
            [CompletionResult]::new('manpages', 'manpages', [CompletionResultType]::ParameterValue, 'Create `man` page files with documentation about all options and subcommands')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'protomd;help;init' {
            break
        }
        'protomd;help;templates' {
            break
        }
        'protomd;help;schema' {
            break
        }
        'protomd;help;completion' {
            break
        }
        'protomd;help;manpages' {
            break
        }
        'protomd;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
