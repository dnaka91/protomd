# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_protomd_global_optspecs
	string join /n I/include= o/output-dir= clean h/help
end

function __fish_protomd_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_protomd_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_protomd_using_subcommand
	set -l cmd (__fish_protomd_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c protomd -n "__fish_protomd_needs_command" -s I -l include -d 'Directories that will be searched for referenced schema files' -r -f -a "(__fish_complete_directories)"
complete -c protomd -n "__fish_protomd_needs_command" -s o -l output-dir -d 'Directory to write the generated files into' -r -f -a "(__fish_complete_directories)"
complete -c protomd -n "__fish_protomd_needs_command" -l clean -d 'Remove any content from the output directory before writing any files to it. This is not done when the output directory points to the current directory'
complete -c protomd -n "__fish_protomd_needs_command" -s h -l help -d 'Print help (see more with /'--help/')'
complete -c protomd -n "__fish_protomd_needs_command" -a "init" -d 'Initialize a new configuration file under the current working directory'
complete -c protomd -n "__fish_protomd_needs_command" -a "schema" -d 'Print the schema of the template context on STDOUT'
complete -c protomd -n "__fish_protomd_needs_command" -a "completion" -d 'Create shell completion scripts for all supported shells'
complete -c protomd -n "__fish_protomd_needs_command" -a "manpages" -d 'Create `man` page files with documentation about all options and subcommands'
complete -c protomd -n "__fish_protomd_needs_command" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c protomd -n "__fish_protomd_using_subcommand init" -s h -l help -d 'Print help (see more with /'--help/')'
complete -c protomd -n "__fish_protomd_using_subcommand schema" -s h -l help -d 'Print help (see more with /'--help/')'
complete -c protomd -n "__fish_protomd_using_subcommand completion" -s h -l help -d 'Print help (see more with /'--help/')'
complete -c protomd -n "__fish_protomd_using_subcommand manpages" -s h -l help -d 'Print help (see more with /'--help/')'
complete -c protomd -n "__fish_protomd_using_subcommand help; and not __fish_seen_subcommand_from init schema completion manpages help" -f -a "init" -d 'Initialize a new configuration file under the current working directory'
complete -c protomd -n "__fish_protomd_using_subcommand help; and not __fish_seen_subcommand_from init schema completion manpages help" -f -a "schema" -d 'Print the schema of the template context on STDOUT'
complete -c protomd -n "__fish_protomd_using_subcommand help; and not __fish_seen_subcommand_from init schema completion manpages help" -f -a "completion" -d 'Create shell completion scripts for all supported shells'
complete -c protomd -n "__fish_protomd_using_subcommand help; and not __fish_seen_subcommand_from init schema completion manpages help" -f -a "manpages" -d 'Create `man` page files with documentation about all options and subcommands'
complete -c protomd -n "__fish_protomd_using_subcommand help; and not __fish_seen_subcommand_from init schema completion manpages help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
