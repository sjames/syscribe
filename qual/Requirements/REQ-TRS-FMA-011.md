---
id: REQ-TRS-FMA-011
type: Requirement
name: feature-check --deep shall optionally emit externally-checkable DRAT proofs for void and dead-feature findings
status: draft
reqDomain: software
verificationMethod: test
---

For tool qualification it is valuable to back an UNSAT-based verdict with **independently checkable evidence**: not just *"the tool says the model is void"* but *"…and here is a proof a third party can verify."* batsat can emit DRAT proofs in-library; the tool **shall** expose this.

### Behaviour

- With an opt-in flag (`feature-check --deep --prove <dir>`), for each UNSAT-based finding the tool **shall** write into `<dir>` the **DIMACS CNF** of the exact formula whose unsatisfiability the finding asserts — `Φ` for `E223` (void, file `void.cnf`), `Φ ∧ F` for each `E224` (dead feature `F`, file `dead-<F>.cnf`).
- Proofs are **off by default** — no files are written unless `--prove` is given.
- The emitted CNF **shall** correspond exactly to the asserted formula (same variable mapping) and be well-formed DIMACS, so an external solver can independently re-confirm it is **UNSAT**. External verification is outside the tool's process and is a documented step.
- File contents **shall** be deterministic for a fixed model ([[REQ-TRS-FMA-006]]); Boolean layer only.

> **Implementation note.** A DIMACS CNF that re-checks as UNSAT is the *evidence* emitted today. The stronger **DRAT refutation proof** is **deferred**: batsat 0.6.0 does not expose a solver-recorded DRAT proof from `solve` (its `drat::Proof` is an unused builder). Emitting a true DRAT proof would require a proof-recording solver (e.g. varisat's proof API, or a batsat fork), tracked separately.

### Scope note

DRAT certifies **UNSAT** results (void, dead). SAT-based results (core/false-optional/configuration validity) are not DRAT-certifiable and are out of scope for this requirement.

**Source:** ADR-FM-002 (batsat in-library DRAT).

**Acceptance criteria:** With `--prove`, a void model writes a non-empty, well-formed DIMACS CNF (`void.cnf`) for `Φ`; a model with a dead feature writes a `dead-<F>.cnf` for `Φ ∧ F`. Without `--prove`, no proof files are written. The emitted DIMACS, solved independently, is UNSAT (self-consistency). Emitted files are byte-identical across two runs. (DRAT-proof emission is deferred per the implementation note.)
