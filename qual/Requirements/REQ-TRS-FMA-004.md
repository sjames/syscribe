---
id: REQ-TRS-FMA-004
type: Requirement
name: Configuration Validity Under Full Feature-Model Semantics
title: feature-check --deep shall verify each authored Configuration is a valid model of the feature model
status: draft
reqDomain: software
verificationMethod: test
---

The extensional pass checks only `requires`/`excludes` over selected features (`E219`/`E220`). It does **not** check group, cardinality, mandatory, or parent-selection semantics. `feature-check --deep` **shall** close this gap by checking every authored `Configuration` against the full encoding ([[REQ-TRS-FMA-001]]).

### Check

- The selection of a `Configuration` is the assignment that sets each feature in its `features:` map to the declared Boolean and treats every other feature as **not selected** (closed-world; the spec expects all features to appear — §9.8).
- If that assignment is **not a model** of the encoded formula `Φ`, the tool **shall** emit `E225` against the `Configuration`, naming the specific violated obligation where determinable:
  - a `mandatory` feature not selected while its parent is selected;
  - an `alternative` group with not exactly one (or not within `cardinality`) selected child while the group parent is selected;
  - an `or` group with a child count outside its `cardinality` while the parent is selected;
  - a feature selected whose parent is not selected;
  - (`requires`/`excludes` violations continue to be reported as `E219`/`E220` by the extensional pass; `E225` **shall not** duplicate them).

### Notes

- `E225` is reported per offending `Configuration` (one finding may summarise multiple violations, or one per violation — implementation choice — but each distinct violation class present **shall** be represented).
- A `Configuration` that is a valid model **shall** produce no `E225`.

**Source:** ADR-FM-001; spec §9.6 (group semantics), §9.8 (`Configuration.features`).

**Acceptance criteria:** A `Configuration` that omits a mandatory feature, selects two children of an `alternative` group, violates an `or`-group `cardinality`, or selects a child without its parent each yields an `E225` naming the relevant feature/group; a fully valid `Configuration` yields no `E225`; a `requires`/`excludes` violation is reported as `E219`/`E220` and is not also emitted as `E225`.
