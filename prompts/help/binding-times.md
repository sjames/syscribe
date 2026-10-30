# binding-times — list the effective binding-time stage vocabulary

## SYNOPSIS
    syscribe -m <root> binding-times [--json]

## DESCRIPTION
Lists the ordered stages a `bindingTime:` may name (ADR-SYS-BT-001), earliest first:
the built-in `compile < load < runtime`, refined by `[bindingTimes.<name>]` tables in
`<model_root>/.syscribe.toml`. Read-only.

Run this before authoring a `bindingTime:` — the vocabulary is per-project, and a
`bindingTime:` that is not one of these stages is error `E230`. An entry reported as
`W750` (malformed declaration) is ignored and not listed.

Declaration keys:
    description   prose
    binder        free label for who resolves values at this stage
    after         an existing stage this one follows
    before        an existing stage this one precedes
At least one of `after`/`before` is required. Two stages that would sit in the same
gap (e.g. both `after = "compile"`) are both rejected as W750 — order them by naming
one as the other's `after`/`before`. A project can insert stages but never
rename or reorder the three built-in ones.

## OPTIONS
    --json   {"bindingTimes": [{name, order, description, binder, builtin}]}

## EXAMPLES
    syscribe -m model/ binding-times
    syscribe -m model/ binding-times --json

## SEE ALSO
    feature, validate, mcp
