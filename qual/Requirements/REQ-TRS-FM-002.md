---
id: REQ-TRS-FM-002
type: Requirement
name: Feature Model Structural Integrity
title: feature-check shall validate requires/excludes resolution and satisfaction, and flag dead or always-on optional features
status: draft
reqDomain: software
verificationMethod: test
---

The `feature-check` command ([[REQ-TRS-FM-001]]) **shall** enforce the following feature-model structural rules:

| Code | Condition |
|---|---|
| `E212` | A `FeatureDef.requires:` or `FeatureDef.excludes:` entry does not resolve to another `FeatureDef` |
| `E219` | In some `Configuration`, a **selected** feature's `requires:` target is **not** selected (cross-tree implication violated) |
| `E220` | In some `Configuration`, a **selected** feature's `excludes:` target **is** also selected (mutual exclusion violated) |
| `W011` | A `FeatureDef` with `groupKind: optional` is selected in **zero** `Configuration` files (possible dead feature) |
| `W012` | A `FeatureDef` with `groupKind: optional` is selected in **every** `Configuration` (should likely be `mandatory`) |

Selection state for a feature is read from each `Configuration.features:` map (a feature absent from the map is treated as not selected). `E219`/`E220` are reported against the offending `Configuration`; `E212` against the declaring `FeatureDef`; `W011`/`W012` against the `FeatureDef`.

**Out of scope (not yet implemented):** group-cardinality rules that require reconstructing the group tree (`E216` mandatory omitted, `E217` alternative both-selected, `E218` `or` cardinality) and two-level satisfies completeness (`E210`/`E211`).

**Source:** §9.6, §9.11.

**Acceptance criteria:** For each of `E212`, `E219`, `E220`, `W011`, and `W012`, a crafted feature model triggering exactly that condition produces a finding with that code under `feature-check`; a clean feature model produces none of them.
