---
type: Requirement
id: REQ-TRS-SYSMLV2-005
name: "A SysMLv2 variation point can target a Syscribe FeatureDef via a @SyscribeFeature metadata annotation"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - variability
---

A SysMLv2 `variation`/`variant` element shall be able to declare a `@SyscribeFeature { featureId =
'<FEAT-id>'; }` metadata annotation. The mapper lifts `featureId` into the synthesized element's
feature-model gate — the same form Syscribe's `appliesWhen:`-consuming code already reads — so the
existing feature-model/SAT engine (`batsat`, `feature-check --deep`, `configure`) reasons about
SysMLv2-authored variation points identically to native ones, with no changes to the solver.

## Rationale

Variability/feature-model semantics (cross-tree constraints, SAT-backed configuration validity)
are a Syscribe invention with no equivalent construct in vanilla SysML v2. A metadata annotation is
SysML v2's own standards-compliant extension point for exactly this kind of tool-specific
attachment — a real, structurally parseable AST node — rather than a fragile comment-based
convention that a formatter or another tool could silently mangle.

## Scope

- A `variation`/`variant` element with no `@SyscribeFeature` annotation is ingested normally as a
  structural element; it simply doesn't participate in the feature-model graph, same as a native
  element with no `appliesWhen:`.
- An unresolvable `featureId` (no matching `FeatureDef`) is a dangling-reference finding, the same
  class already raised for any other unresolved feature reference today.
- This requirement does not extend feature-model semantics themselves (multi-feature expressions,
  cross-tree constraints authored from the SysMLv2 side, etc.) — a single `featureId` reference is
  the full scope of this requirement.
