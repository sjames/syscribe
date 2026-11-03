# log — the history of the model, as git records it

## SYNOPSIS
    syscribe -m <root> log [<element>] [--by <actor>] [--since <date>] [--unattributed]
                           [--limit N] [--json]

## DESCRIPTION
There is no separate change log: git is the record. A writing command run with `--commit` (or with
`[provenance] commit = true` in `.syscribe.toml`) commits the files it changed — only those — with
`--by <actor>` as the commit author and trailers carrying the command, actor and reason:

    syscribe: set REQ-X status=approved

    evidence TC-3 passed

    Syscribe-Command: set REQ-X status=approved
    Syscribe-Actor: agent-1
    Syscribe-Reason: evidence TC-3 passed

`log` reads that history back.

- **`log`** lists the commits that touched the model, newest first, with the number of files each
  changed.
- **`log <element>`** follows one element's file (through renames) and says what each commit changed
  in it: the status transition, links and evidence added or removed, other fields, the body.
- **`--by <actor>`** keeps commits made by that actor (the commit author or a `Syscribe-Actor`).
- **`--since <date>`** keeps commits after a date (anything `git log --since` accepts).
- **`--unattributed`** keeps the commits that touched the model without syscribe's trailers — edits
  made by hand, or by a tool that was not told to commit.

Uncommitted changes to the model are not in git yet; they are listed at the top so they are not
mistaken for missing history. A model outside a git repository exits `1`.

## OPTIONS
    --by <actor>      Only commits by this author or actor.
    --since <date>    Only commits after this date.
    --unattributed    Only commits without Syscribe-* trailers.
    --limit N         At most N commits (default 50).
    --json            Machine-readable output (schemaVersion 1.0).

## EXAMPLES
    syscribe -m model/ log --limit 10
    syscribe -m model/ log REQ-TRS-PIQ-001
    syscribe -m model/ log --unattributed --since 2026-11-01 --json

## SEE ALSO
    changes, set, claim, validate
