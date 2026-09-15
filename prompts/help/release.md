# release — clear a PlanningItem's claim marker

## SYNOPSIS
    syscribe -m <root> release <PI-id> [--dry-run]

## DESCRIPTION
Clears `claimedBy:`/`claimedAt:` on a `PlanningItem` — e.g. on completion or
handoff to another agent. Clears both fields regardless of the item's current
`status:`. A no-op (nothing written) when the item is not currently claimed.

## OPTIONS
    --dry-run   Preview the unified diff without writing.

## EXAMPLES
    syscribe -m model/ release PI-HPLE-001
    syscribe -m model/ release PI-HPLE-001 --dry-run

## SEE ALSO
    claim, set, show
