---
type: Requirement
id: REQ-TRS-BL-011
name: "Configuration-projected baseline scope"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BL-000]
breakdownAdr: Decisions::BaselineADR
tags:
  - baseline
  - scope
  - variability
---

A baseline shall be able to freeze a **projected product-line variant**, not only the flat
model, so a specific configured build can be sealed and assessed (REQ-TRS-BL-003 reserved
this for a later phase).

- `frozenScope.config` shall name a `Configuration` (by id or qualified name), or an ad-hoc
  comma-separated `FeatureDef` set, resolved by the standard selection resolver.
- When `frozenScope.config` is set, scope resolution shall first **project** the model to the
  selected variant (dropping every element inactive under that selection, per the variability
  `appliesWhen` semantics), and then apply the remaining `frozenScope` filters
  (`package`/`types`/`status`/`tags`) and the `Baseline`-exclusion over the **projected**
  element set. The seal, `elementCount`, and manifest therefore describe exactly the variant.
- `create --frozen-scope "config=CONF-ABS;status=approved"` shall accept `config` as a scope
  clause. A `config` that does not resolve to a Configuration or feature set shall cause
  `create` to refuse and write nothing.
- Drift detection (REQ-TRS-BL-005) and `verify` (REQ-TRS-BL-008) shall re-project with the
  stored `config` before recomputing the aggregate, so that a change to the variant's active
  content — including a change to the named `Configuration`'s feature selections — is detected
  as drift.
- On a model with **no feature model**, a `config` clause is inert and the scope behaves as an
  unprojected (whole-model / filtered) baseline, so single-configuration models are unaffected.

Trace-closure scope (`closureFrom`) remains out of scope and reserved for a later phase.
