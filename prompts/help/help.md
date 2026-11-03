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

## GLOBAL WRITE OPTIONS
Accepted by every command, before or after its name:

    --dry-run              Any command that writes files leaves the tree untouched and says what it
                           would have written. A status line reads "Would write", never "Wrote".
    --by <actor>           Who is making the change (letters, digits, '.', '_', '-'). Optional. With
                           --commit it is the commit author. (`claim` and `log` give --by their own
                           meaning: the claimant, and a filter.)
    --reason <text>        Why — recorded in the commit message.
    --commit               Commit the files this command wrote, and only those, so another actor's
                           pending work in the same tree is never swept in. Also on when
                           `[provenance] commit = true` in .syscribe.toml. Never on a dry run.
    --as-human             Declare that a person is making this change, so the project's agent policy
                           (`[agents]` in .syscribe.toml, see `policy`) does not apply to it. A declaration,
                           not a proof; recorded as a `Syscribe-As: human` trailer when committed.
    --expect-hash <hash>   Refuse (exit 1, nothing written) if the target file's content hash is not
                           this one — the `contentHash` from `show` or the MCP `get_element` detail.
                           Accepted by set, claim, release, move, applies-when and scaffold-gherkin --fix;
                           anywhere else it is a usage error, never silently ignored. It is a check before
                           the write, not a lock.

Git is the only record of who changed what: a committed change carries `Syscribe-Command`,
`Syscribe-Actor` and `Syscribe-Reason` trailers, and `syscribe log` reads them back.

Exit codes: 0 success; 1 a usage error, an unknown element or a refused write; 2 a tripped gate.
`connectivity`, `follow` and `impact` also exit 2 on a usage error.

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
