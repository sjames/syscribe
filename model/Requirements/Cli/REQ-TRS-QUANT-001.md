---
type: Requirement
id: REQ-TRS-QUANT-001
name: "quantities carries structured timing values and validate checks their budgets"
status: draft
reqDomain: software
reqClass: system
tags:
  - safety
---

An element shall be able to state FTTI, latency, WCET and reaction time as structured values, and `validate` shall check that a chain fits its budget (GH #237, v1).

## Behavior

- `quantities:` (a list, on any element — in practice `SafetyGoal`, `Requirement`, `TestCase`) of mappings `{kind, value, unit}`: `kind` one of `ftti`, `latency`, `wcet`, `reaction`; `value` a positive number; `unit` one of `s`, `ms`, `us`, `ns`. A `SafetyGoal`'s existing `ftti:` string counts as an `ftti` quantity.
- `E895` — an entry that is not a mapping, has an unknown `kind` or `unit`, or a non-positive / non-numeric `value`.
- `W893` — a budget is exceeded: (a) for a `Requirement` or `SafetyGoal` with a quantity of kind *K*, the sum of kind *K* over the requirements derived from it (`derivedFrom:` for a requirement, `derivedFromSafetyGoal:` for a goal), treated as a serial chain, is larger than its value; (b) for a `SafetyGoal` with an `ftti`, a derived requirement whose `latency` plus `reaction` quantities exceed it. Units are normalised before comparing. A parent without a child quantity of the kind raises nothing. Draft elements are not checked.
