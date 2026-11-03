# help — detailed per-command help

## SYNOPSIS
    syscribe help [<command> | <Type> | tasks | <code>]
    syscribe help --search <keyword>      (also: help -k <keyword>)
    syscribe <command> --help | -h

## DESCRIPTION
With no argument, lists every command with a one-line summary. With a command,
prints its detailed man page (synopsis, options, examples, exit codes, see-also).
`syscribe <command> --help` (or -h) prints the same page. These work without a
model directory.

**By element type.** `help <Type>` (any element type, case-insensitive) prints its core
fields with required/optional and enum domains, a skeleton, the validation codes that name
it, and every command that reads or changes it — `help PlanningItem` lists `claim`,
`release`, `set`, …. The command list is derived from the man pages, so it cannot go stale.

**By keyword.** `help --search <keyword>` ranks command names, synopses, option text, type
names and spec sections, and prints the matching line of each.

**By job.** `help tasks` maps common intents (create safely, record evidence, check a change,
claim work, compare variants, gate CI) to commands. `help <code>` (e.g. `help E106`) is
`explain <code>`.

A mistyped command or topic lists the three nearest commands with their purpose.

## EXAMPLES
    syscribe help
    syscribe help validate
    syscribe help PlanningItem
    syscribe help --search evidence
    syscribe help tasks
    syscribe validate --help
    syscribe matrix -h

## SEE ALSO
    explain, spec, template, --agent-instructions
