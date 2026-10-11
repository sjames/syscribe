---
type: Requirement
id: REQ-TRS-STYPE-002
name: "Type and unit errors in the expressions of executable action flows are found at validation time"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - expressions
---

The static type and dimension check of REQ-TRS-STYPE-001 shall also cover executable `ActionDef`s (GH #272 backlog): `validate`, `compile` and `simulate` report `E935` for an expression of an action flow that the evaluator would reject whatever the values.

## Behavior

- **Types.** A variable, and a parameter that has a default value, takes the type of its declared value (after its `unit`); a parameter without one, a pin of a sub-action (`<sub>.<pin>`), an `as` binding of an accept action and any other name are unknown and never an error.
- **Where.** Succession guards and `condition:` of `IfAction` and `LoopAction` must be Boolean; the right-hand side of every `assign:` must have the kind and dimension of the slot assigned; the argument expressions of `perform` and `send` must type-check; nested blocks are checked like top-level nodes; every definition reached by `perform` is checked in its own scope.
- **Same rules as machines** (`docs/design/static-typing.md`); the same differential guarantee. `E935` is reported together with any other lowering diagnostic of the action.
- **No new false positives.** Existing action examples, models and documented snippets validate unchanged.
- **Not in v1:** dimension inference through pins and through the flows between actions' parameters.
