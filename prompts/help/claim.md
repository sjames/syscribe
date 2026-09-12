# claim — advisory ownership marker on a PlanningItem

## SYNOPSIS
    syscribe -m <root> claim <PI-id> --by <agent-id> [--dry-run]

## DESCRIPTION
Sets `claimedBy:`/`claimedAt:` on a `PlanningItem` — a coordination signal for
running several agents against one model concurrently ("is anyone already on
this?"), not a filesystem lock. Refuses (no file written) when the item is
already `status: done` (nothing to claim), or already claimed by a *different*
`--by` value; re-claiming with the same `--by` is allowed and refreshes
`claimedAt:`.

`claimedBy`/`claimedAt` are visible in `show <PI-id>` and in
`list PlanningItem --json`. Two simultaneously-active (`in_progress` or
claimed) `PlanningItem`s that overlap by `achieves:` or an `evidence[].path`
raise `W311` on the next `validate` — the same signal, surfaced automatically.

## OPTIONS
    --by <agent-id>   Required. Opaque claimant id (an agent/session id).
    --dry-run         Preview the unified diff without writing.

## EXAMPLES
    syscribe -m model/ claim PI-HPLE-001 --by agent-session-01VRUS
    syscribe -m model/ claim PI-HPLE-001 --by agent-session-01VRUS --dry-run

## SEE ALSO
    release, set, show
