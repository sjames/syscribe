# steps — a PlanningItem's breakdown as a tree with a roll-up

## SYNOPSIS
    syscribe -m <root> steps <PI> [--open] [--depth N] [--json]

## DESCRIPTION
A step is a child `PlanningItem` (`parent: <PI>`), so this adds a *view*, not a field. It renders
the item's children recursively with each node's status, `assignedTo`, `claimedBy`, what it is
`blockedBy`, the number of open questions (`?N`) and, for a `done` leaf, whether it has any
evidence. The header gives the roll-up: `N of M leaves done` and the counts still in progress,
blocked or not started.

The parent's status is not derived from its children and no rule ties them together. Where a
composite item disagrees with its leaves (all leaves done but the item is not; or the item is
`done` with unfinished leaves) the view says so under *Notes (not findings)*; `validate` is
unaffected.

## OPTIONS
    --open        Hide subtrees that are entirely done (the roll-up still counts them).
    --depth N     Show N levels below the item; deeper levels are summarised.
    --json        The tree, roll-up and notes as JSON (schemaVersion 1.0).

An unknown item, or one that is not a PlanningItem, exits 1.

## EXAMPLES
    # against the bundled PlanningItem example (examples/planning-item/model/)
    syscribe -m examples/planning-item/model/ steps PI-RTH-001
    syscribe -m examples/planning-item/model/ steps PI-RTH-001 --open --depth 2

## SEE ALSO
    questions, claim, set, show
