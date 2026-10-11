---
type: Requirement
id: REQ-TRS-SMSCHEMA-001
name: "A state machine can declare typed variables, time and change triggers, internal transitions and assignments"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - state-machine
---

The `StateDef` schema of §8.8 shall carry what an interpreter needs to run a machine, and the validator shall check it (GH #275, part of #272). Everything is optional and additive: a model that uses none of it validates as before.

## Behavior

- **`variables:`** on a `StateDef` (or `State`) declares the machine's data: a list of `{name, typedBy?, unit?, value}`. `name` is a basic name (`^[A-Za-z_][A-Za-z0-9_]*$`) and unique in the machine; `value` is an expression of REQ-TRS-EXPR-001 evaluable with no names, optionally read with `unit:` / an `ISQ::…Value` type as in REQ-TRS-PARAM-001. `E883` — a malformed list, an invalid or duplicate name, or an initial value that does not evaluate.
- **Triggers.** A transition's `accept:` may be a payload (`Pkg::Cmd`, or `{payload, via, as}` as today) or exactly one of `{after: <duration>}` (fires that long after the source state was entered, the timer being cancelled when the state is left), `{when: <boolean expression>}` (a SysML v2 change trigger: sent when the expression is, or becomes, true after the source state is entered — see `docs/design/state-machine-execution-semantics.md`) or `{done: <region or state name>}` (fires when that region or composite state has reached a final state). `as:` names the received event's payload so guards and assignments can read `<as>.<field>`. `E882` — more than one trigger kind, an unknown trigger key, `as`/`via` without a `payload`, an `after` that does not parse or whose constant value is not a duration, a `when` that does not parse, or a `done` naming no region or state of the machine.
- **Internal transitions.** `internal: true` marks a transition that runs its `effect:`/`assign:` without leaving or re-entering its source state; it carries no `target:`. `E885` — `internal` is not a boolean, or an internal transition has a `target:`. `W929` no longer asks an internal transition for a target. An internal transition is not an outgoing edge for `W070`/`W071`.
- **Assignments.** `assign:` (on a transition, and in `entryAction`/`exitAction`/`initialEffect` maps) is a `;`-separated list of `name := expression` statements in the same language, run after any `effect:`. `E884` — a statement that does not parse, or one that assigns a name that is not a declared variable.
- **`initialEffect:`** on a `StateDef` or composite substate (`{effect?, assign?}`) runs when the region is entered through its initial state.
- **Executable machines.** A machine is *executable* when it declares `variables:` or uses `after`, `when`, `done`, `internal`, `assign` or `initialEffect`. Only for those, `W933` reports a `guard:` or `when:` that is not in the expression language (it would stop the machine from being simulated) and a name in one that is neither a declared variable, an `as` binding nor a model attribute path. Opaque guards of non-executable machines stay unchecked.
- The authoring rules, with an example, are in `docs/model-guide/state-machines.md`; the codes are in the validation catalogue.
