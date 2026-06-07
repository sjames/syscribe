---
id: REQ-TRS-FMA-006
type: Requirement
name: Decision Procedure — Determinism, Reproducibility, Bounds, and Scope
title: Deep analysis shall use a deterministic pure-Rust decision procedure, guard against blow-up, and stay within the Boolean layer
status: draft
reqDomain: software
verificationMethod: test
---

Because the tool is qualification-targeted (ISO 26262-8 §11 / IEC 61508-3), the deep analysis's decision procedure carries non-functional obligations that are as binding as the analyses themselves.

### Determinism

- For identical inputs, the deep analysis **shall** produce **identical** findings, explanations, and report ordering across repeated runs and across supported platforms. The decision procedure **shall** therefore be canonical or made deterministic by construction (fixed variable ordering, fixed iteration order — see [[REQ-TRS-FMA-001]]).

### Reproducible builds / dependencies

- The decision procedure **shall** be a **vendored pure-Rust** library: **no external solver process**, no native/system library, and no network access. (ADR-FM-002 selects `batsat` — a pinned, MIT, MiniSat-derived pure-Rust CDCL solver — as the sole engine; there is no in-tree solver.)

### Bounded execution

- The deep analysis **shall** guard against pathological blow-up. When a model exceeds a documented size threshold (feature count and/or decision-diagram node budget), the tool **shall** emit a clear diagnostic and **skip** the deep analysis gracefully (without hanging or exhausting memory) rather than attempt an unbounded computation. The threshold **shall** be documented and **should** be overridable by an explicit flag.
- A skipped deep analysis **shall not** be reported as a sound "model OK"; the diagnostic **shall** make the skip explicit.

### Scope boundary

- The deep analysis covers the **Boolean feature layer only**. The following are explicitly **out of scope** and deferred to a future SMT/CSP requirement:
  - numeric parameter satisfiability (`range:`, `isArray:` element ranges),
  - `derivedFrom:` arithmetic *evaluation* (only cycle detection, `E207`, is in scope today),
  - `parameterConstraints` expression *evaluation* to a Boolean (`E221`; only path resolution `E213` and `appliesWhen` presence `W014` exist today).
- Output **shall not** imply that parameter-level satisfiability has been checked.

**Source:** ADR-FM-001; tool-qualification determinism/reproducibility obligations.

**Acceptance criteria:** Running the deep analysis twice on the same model yields byte-identical findings and explanations. A build of the tool requires no external solver binary or network access. A synthetic model exceeding the configured size threshold produces an explicit "deep analysis skipped" diagnostic and a non-hanging exit, not a false "OK". Documentation and `--json` clearly scope the result to the Boolean layer.
