---
type: Requirement
id: REQ-TRS-ACTSCHEMA-001
name: "An ActionDef can declare variables, expression guards and conditions, assignments and timed accepts, and the validator checks them"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - action-flow
---

The `ActionDef` schema of §8.7 shall carry what an interpreter needs to run an action flow, and the validator shall check it (GH #276, part of #272). Everything is optional and additive: a model that uses none of it validates as before.

## Behavior

- **`variables:`** on an `ActionDef` declares local data exactly as on a `StateDef` (REQ-TRS-SMSCHEMA-001): a list of `{name, value, typedBy?, unit?}`. Together with the `in`/`out`/`inout` `parameters:` they are the action's data; an `out` parameter without a value starts unset.
- **Expressions.** The `guard:` of a succession, the `condition:` of an `IfAction` and of a `LoopAction`, and an `AssignmentAction`'s `assign:` (`name := expression; …`) are written in the expression language of REQ-TRS-EXPR-001. A name in them is a parameter, a variable, a payload field bound by an `AcceptAction`'s `as:`, or `<subaction>.<output parameter>`.
- **`AssignmentAction`** carries `assign:`; `E884` (shared with state machines) for an unparseable statement or an undeclared target. `E883` (shared) for a malformed `variables:` list.
- **`AcceptAction`** may carry `after: <duration>` (a timed accept: it completes that long after it became enabled) instead of a `payload`; `as:` names the received payload.
- **Decision nodes** (checked for executable actions only). A `DecisionNode`'s outgoing successions are tried in document order; the first whose `guard:` holds is taken; a succession with `guard: else` is the default. `E887` — more than one `else`, an `else` that is not last, a decision with no outgoing succession, or a `guard:` on a succession that is not in the expression language.
- **Loops.** A `LoopAction` has `loopKind: while|until|for` and a `condition:` (for `for`, `over: <expression>` is not supported and is `E887`); `E887` for a `while`/`until` loop without a `condition:`.
- **Executable actions.** An `ActionDef` is *executable* when it declares `variables:` or any sub-action uses `assign:`. Only for those, `W935` reports a guard or condition that is not in the expression language, or a bare name in one that is neither a parameter, a variable, an `as` binding nor a `<subaction>.<pin>`. Opaque guards of non-executable actions stay unchecked.
