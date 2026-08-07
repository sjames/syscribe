---
type: Requirement
id: REQ-TRS-HPLE-003
name: "A parameterBindings: entry reaching into a consolidated subtree must target an actually-open parameter"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-HPLE-000]
breakdownAdr: Decisions::HierarchicalProductLineADR
tags:
  - variability
  - multi-repo
  - validation
---

A `parameterBindings:` entry reaching into a `subConfigurations:`-consolidated subtree shall target
a parameter that is genuinely **open** at the point it is bound: the parameter's owning `FeatureDef`
is actually selected by the relevant descendant `Configuration` (directly or, transitively, by
whichever nearer tier resolved it), the parameter is not `isFixed: true`, and it is not already
bound by that descendant's own `Configuration` or by a nearer tier's `parameterBindings:` on the
path down to it.

## Rationale

This directly extends the existing single-model rules — a `Configuration` must not bind a parameter
of an unselected feature (`E203`) or one that is `isFixed: true` (`E204`) — across the
`subConfigurations:` boundary. Without this, a higher tier could silently inject a value that a
lower tier never asked for and has no obligation to honor, or double-bind something already
resolved, undermining the "consolidation of already-configured models" guarantee `REQ-TRS-HPLE-001`
establishes.

## Scope

- Targeting an unselected feature's parameter, a fixed parameter, or a parameter already bound
  somewhere nearer in the chain is a validation error, following the same reasoning as `E203`/`E204`
  extended cross-tier.
- "Already bound by a nearer tier" reflects the transitive-deferral model this feature supports: the
  same parameter may legally be closed by any one tier along the path from where it's declared open
  up to wherever it's finally supplied — it is only illegal to supply it *twice*.
- Determining which parameters remain genuinely open, in aggregate, across an entire consolidated
  subtree is `REQ-TRS-HPLE-004`'s completeness check, not this requirement — this requirement
  covers rejecting an individual illegal binding, not reporting what's still missing.
