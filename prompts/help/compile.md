# compile — lower a state machine to its intermediate representation

## SYNOPSIS
    syscribe compile <StateDef> [--json]

## DESCRIPTION
Lowers a `StateDef` (or `State`) to the machine IR that the simulator, coverage,
mutation and SCXML export all read: a flat state table (ids, parents, kinds,
initial children, entry/exit actions), transitions in document order (event, timer,
change or done trigger, guard, actions), a variable table with slots and initial
values, and a source pointer (element and YAML path) on every node. The BLAKE3
hash of the canonical JSON identifies the machine in recorded evidence.

Lowering refuses what validation rejects: schema errors (E882–E885), a guard or
`when` that is not in the expression language or uses an undeclared name (W933),
an unresolved or ambiguous transition endpoint (W076).

    syscribe compile Door            # summary
    syscribe compile Door --json     # canonical IR

An `ActionDef` is lowered to the action-graph IR instead (nodes, guarded successions, slots, data
flows, called actions): see `syscribe help simulate`.

## EXIT STATUS
0 lowered · 1 usage, or the element is not a state machine · 2 diagnostics.
