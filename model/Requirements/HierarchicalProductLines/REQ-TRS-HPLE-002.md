---
type: Requirement
id: REQ-TRS-HPLE-002
name: "parameterBindings: reaches transitively into a consolidated subtree via ordinary qname resolution"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-HPLE-000]
breakdownAdr: Decisions::HierarchicalProductLineADR
tags:
  - variability
  - multi-repo
---

The existing `Configuration.parameterBindings:` field (a flat map keyed by the canonical dotted
`<FeatureDef qualified name>.<parameter name>` reference) shall be able to bind a parameter
belonging to any `FeatureDef` reachable through `subConfigurations:` — at any depth, not only this
`Configuration`'s own local features — using the parameter's ordinary, already-mounted qname. No new
field, and no new cross-repo addressing syntax distinct from an ordinary qualified name, is
introduced.

## Rationale

This is the direct, minimal consequence of two already-true facts: `parameterBindings:`'s dotted
key is already just a qname-shaped reference, and a peer's elements are already mounted into the
local namespace by `repoImports:` (transitively — `E510`'s "directly or transitively" language
confirms the existing loader already walks multi-hop import chains). Once both are true, "can a
dotted key reach three tiers down" requires no new mechanism at all, only confirming the existing
resolution path composes correctly end to end.

## Scope

- A `parameterBindings:` entry targeting a parameter reachable only through `subConfigurations:`
  resolves and applies exactly like one targeting a purely local `FeatureDef` parameter.
- This requirement does not itself define which bindings are *permitted* (a parameter must
  actually be open — required, unbound, and belonging to a selected feature — to legally accept an
  injected value) — that is `REQ-TRS-HPLE-003`.
- `bindTo:`'s existing single-model component/system parameter propagation is unaffected and does
  not participate in this resolution path — a `bindTo:` target still resolves purely within its own
  model (`ADR-SYS-HPLE-001`'s explicit non-goal).
