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

### Scale target and bounded execution

- The deep analysis **shall** comfortably analyze feature models of at least **~500 features** within interactive time (seconds) and modest memory — this is the supported working scale.
- It **shall** guard against pathological blow-up: when a model exceeds a documented feature-count limit (default **1000**, comfortably above the ~500 target), the tool **shall** emit a clear diagnostic and **skip** the deep analysis gracefully (without hanging or exhausting memory) rather than attempt an unbounded computation. The limit **shall** be documented and **should** be overridable by an explicit flag.
- A skipped deep analysis **shall not** be reported as a sound "model OK"; the diagnostic **shall** make the skip explicit.

### Scope boundary

- The deep analysis covers the **Boolean feature layer only**. The following are explicitly **out of scope** and deferred to a future SMT/CSP requirement:
  - numeric parameter satisfiability (`range:`, `isArray:` element ranges),
  - `derivedFrom:` arithmetic *evaluation* (only cycle detection, `E207`, is in scope today),
  - `parameterConstraints` expression *evaluation* to a Boolean (`E221`; only path resolution `E213` and `appliesWhen` presence `W014` exist today).
- Output **shall not** imply that parameter-level satisfiability has been checked.

**Source:** ADR-FM-001; tool-qualification determinism/reproducibility obligations.

**Acceptance criteria:** A synthetic model of ~500 features is **analyzed** (not skipped) and completes within interactive time, with correct results. Running the deep analysis twice on the same model yields byte-identical findings and explanations. A build of the tool requires no external solver binary or network access. A model exceeding the documented limit (1000) produces an explicit "deep analysis skipped" diagnostic and a non-hanging exit, not a false "OK". Documentation and `--json` clearly scope the result to the Boolean layer.
