---
id: REQ-TRS-FMA-011
type: Requirement
name: DRAT Proof-Carrying Evidence for UNSAT Findings
title: feature-check --deep shall optionally emit externally-checkable DRAT proofs for void and dead-feature findings
status: draft
reqDomain: software
verificationMethod: test
---

For tool qualification it is valuable to back an UNSAT-based verdict with **independently checkable evidence**: not just *"the tool says the model is void"* but *"…and here is a proof a third party can verify."* batsat can emit DRAT proofs in-library; the tool **shall** expose this.

### Behaviour

- With an opt-in flag (`feature-check --deep --prove <dir>`), for each UNSAT-based finding the tool **shall** write, into `<dir>`:
  - the **DIMACS CNF** of the exact formula whose unsatisfiability the finding asserts — `Φ` for `E223` (void), `Φ ∧ F` for each `E224` (dead feature `F`); and
  - the corresponding **DRAT** unsatisfiability proof produced by the solver.
- Proofs are **off by default** — no files are written unless `--prove` is given.
- The emitted CNF and proof **shall** correspond exactly to the asserted formula (same variable mapping), so an external checker (e.g. `drat-trim`, `gratchk`) can verify them. External verification is outside the tool's process and is a documented step.
- File contents **shall** be deterministic for a fixed model ([[REQ-TRS-FMA-006]]); Boolean layer only.

### Scope note

DRAT certifies **UNSAT** results (void, dead). SAT-based results (core/false-optional/configuration validity) are not DRAT-certifiable and are out of scope for this requirement.

**Source:** ADR-FM-002 (batsat in-library DRAT).

**Acceptance criteria:** With `--prove`, a void model writes a non-empty DIMACS CNF plus a well-formed DRAT proof for `Φ`; a model with a dead feature writes a CNF + proof for `Φ ∧ F`. Without `--prove`, no proof files are written. The emitted DIMACS, solved independently, is UNSAT (self-consistency); where `drat-trim` is available it accepts the proof (documented external check). Emitted files are byte-identical across two runs.
