---
id: REQ-TRS-FM-003
type: Requirement
name: feature-check shall detect derivedFrom cycles, bindTo propagation range violations, and parameterConstraints path errors
status: draft
reqDomain: software
verificationMethod: test
---

The `feature-check` command ([[REQ-TRS-FM-001]]) **shall** enforce the parameter-integrity rules deferred from the single-level binding work ([[REQ-TRS-PARAM-001]]):

| Code | Condition |
|---|---|
| `E207` | Circular `derivedFrom:` dependency between parameters of the **same** `FeatureDef` (parameter A derived from B, B derived from A, directly or transitively) |
| `E202` | A value propagated via `bindTo:` — a component parameter bound to a system parameter (`bindTo:` target written in the canonical dotted form `<FeatureDef>.<param>`, see [[REQ-TRS-PARAM-002]]) that a `Configuration` sets — falls **outside** the component parameter's narrowing `range:` |
| `E213` | A cross-feature `parameterConstraints` expression references a parameter path (`<FeatureDef>.<param>`, the canonical dotted form) that does not resolve to a declared parameter |
| `W014` | A `parameterConstraints` entry has an `appliesWhen:` referencing a `FeatureDef` that is selected in **no** `Configuration` |

`parameterConstraints:` are read from package `_index.md` frontmatter (the feature-model or configuration authority, §9.7). `derivedFrom:` and `expression:` are opaque strings; the tool resolves the qualified parameter paths and sibling parameter names they reference.

**Out of scope (not yet implemented):** evaluating a `parameterConstraints` expression to a boolean (`E221`) — this needs an expression evaluator.

**Source:** §9.7, §9.9, §9.11.

**Acceptance criteria:** For each of `E207`, `E202`, `E213`, and `W014`, a crafted feature model triggering exactly that condition produces a finding with that code under `feature-check`; a feature model with an acyclic derivation, an in-range propagated value, resolvable constraint paths, and constraint features present in some configuration produces none of them.
