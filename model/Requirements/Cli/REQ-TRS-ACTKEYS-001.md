---
type: Requirement
id: REQ-TRS-ACTKEYS-001
name: "A sub-action key its kind does not define, and a payload that is not a type name, are reported"
status: implemented
reqDomain: software
reqClass: system
tags:
  - action-flow
  - validation
---

`validate` shall raise `W938` for a sub-action of an `ActionDef` or `Action` that carries a key its kind does not define, instead of ignoring it silently (GH #508).

- **Keys by kind.** `SendAction`: `payload`, `via`, `to`, `parameters`. `AcceptAction`: `payload`, `via`, `to`, `trigger`, `after`, `as`. `AssignmentAction`: `assign`, `target`, `referent`, `value`, `valueKind`. `IfAction`: `condition`, `then`, `else`. `LoopAction`: `loopKind`, `condition`, `body`, `variable`, `sequence`. `PerformAction`/`Action`: `typedBy`, `parameters`, `subActions`. Every kind also takes `name`, `kind`, `doc`, `multiplicity`. A kind the check does not know is left alone.
- **Payload values.** The values of a send's payload fields are bound with `parameters: [{name, value}]`, each an expression evaluated when the send fires. A mapping given as `payload:` or the keys `arguments`, `args`, `values`, `fields`, `attributes` are not fields of a `SendAction`; the finding names the working form.
- **Nested.** The check descends into `then`, `else`, `body` and nested `subActions`.
