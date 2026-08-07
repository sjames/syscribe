---
type: Requirement
id: REQ-TRS-PLANITEM-004
name: "A PlanningItem can be gated to a product-line feature via the existing appliesWhen: mechanism"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLANITEM-000]
breakdownAdr: Decisions::PlanningItemADR
tags:
  - planning
  - variability
---

A `PlanningItem` shall accept the existing, universal `appliesWhen:` field, so a `PlanningItem`
representing the work to implement a product-line feature is included/excluded by
`feature-check --deep`, `configure`, and `validate --config`/`--all-configs` exactly like a native
architecture element's gate.

## Rationale

`appliesWhen:` is already type-agnostic: `feature_model.rs`'s `W014`/`W021`/`W022`/`E227` checks,
`projection.rs`'s `is_active`, and `validator.rs`'s `E209` block all read it off any `RawElement` by
field alone, with zero type or origin filtering anywhere (independently confirmed during the
SysMLv2 `@SyscribeFeature` work, `ADR-SYS-SYSMLV2-001`). This requirement adds no new mechanism —
it only needs `PlanningItem` to be recognised as a valid element to carry the field (which follows
automatically from being a normal `RawElement`) and test coverage proving the existing engine
treats it identically to every other gated element.

## Scope

- No changes to `feature_model.rs`, `projection.rs`, `solver.rs`, or `variability.rs` are expected
  to be necessary; this requirement is satisfied by confirming (with a test, not by inspection
  alone) that a `PlanningItem` with `appliesWhen:` projects in and out correctly across
  `Configuration`s, and is not incorrectly flagged an orphan-feature reference.
- If testing surfaces a gap where `appliesWhen:` is in fact filtered by element type somewhere
  unexpected, closing that gap is in scope for this requirement (it would mean the "already
  type-agnostic" premise was wrong for this specific new type, not that a new mechanism should be
  built).
