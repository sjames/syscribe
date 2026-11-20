# next — the PlanningItem an agent should take next, with the reasons for the rest

## SYNOPSIS
    syscribe -m <root> next [--limit N] [--json]

## DESCRIPTION
Answers "what should I work on?" for any agent harness in one call (GH #485). It offers the `PlanningItem`s that are
`todo` or `in_progress`, unclaimed, a leaf (a composite is worked through its steps), with every `blockedBy:` met and
no open question reserved to a person. Work that was started and left (`in_progress`, unclaimed) comes first, then
items by id. Each candidate lists its hazards: an open question an agent may answer, and overlap with active work by
a shared `achieves:` requirement (`W311`). Everything not offered is listed with the reason: claimed by whom, blocked
by what, waiting on a person, composite, or a status such as `blocked`.

A blocker is met when a PlanningItem is `done`, an ADR is `accepted`, a Requirement is `implemented` or `verified`;
an unresolved `blockedBy:` entry holds the item back.

The command only reads. To take the item: `claim <PI> --by <agent>`; for its briefing: `context <PI>`.

## OPTIONS
    --limit N   How many candidates to list (default 5).
    --json      `{schemaVersion, next, candidates, candidateCount, skipped}`; `next` is null when nothing is offered.

Exit `0` when something is offered, `2` when nothing is, `1` on a usage error.

## EXAMPLES
    # against the bundled PlanningItem example (examples/planning-item/model/)
    syscribe -m examples/planning-item/model/ next
    syscribe -m examples/planning-item/model/ next --json --limit 1

## SEE ALSO
    done, claim, release, context, steps, questions
