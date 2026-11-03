# move — rename/relocate an element, rewriting references

## SYNOPSIS
    syscribe -m <root> move <src> <dest> [--dry-run]

## DESCRIPTION
Moves an element or package to a new qualified name, rewriting every cross-
reference to it across the model. Use --dry-run first to preview the changes.

For id-identified types (Requirement, TestCase, ADR, ...) `move` only relocates the
file: the `id:` inside is not changed, and references resolve by id, so the destination
should keep the same id (the file stem). Nothing stops a move to a qualified name whose
last segment differs from the id; check `validate` afterwards.

## OPTIONS
    --dry-run   Show what would change without writing.
    --json      One result envelope instead of text: {ok, dryRun, command, wrote, wouldWrite, commit, messages, diff}.

Also accepts the global write options `--by`, `--reason`, `--commit`, `--expect-hash` (see `syscribe help`).

## EXAMPLES
    syscribe -m model/ move UAV::Avionics::FlightController UAV::Avionics::FlightComputer --dry-run
    syscribe -m model/ move UAV::Payload UAV::Mission::Payload

## SEE ALSO
    refs, links
