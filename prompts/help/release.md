# release — clear a PlanningItem's claim marker

## SYNOPSIS
    syscribe -m <root> release <PI-id> [--dry-run]

## DESCRIPTION
Clears `claimedBy:`/`claimedAt:` on a `PlanningItem` — e.g. on completion or
handoff to another agent. Clears both fields regardless of the item's current
`status:`. A no-op (nothing written) when the item is not currently claimed.

## OPTIONS
    --dry-run   Preview the unified diff without writing.
    --json      One result envelope instead of text: {ok, dryRun, command, wrote, wouldWrite, commit, messages, diff}.

Also accepts the global write options `--by`, `--reason`, `--commit`, `--expect-hash` (see `syscribe help`).

## EXAMPLES
    # against the bundled PlanningItem example (examples/planning-item/model/)
    syscribe -m examples/planning-item/model/ release PI-RTH-DOCS-001
    syscribe -m examples/planning-item/model/ release PI-RTH-DOCS-001 --dry-run

## SEE ALSO
    claim, set, show
