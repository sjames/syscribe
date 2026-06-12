---
id: REQ-TRS-FMA-010
type: Requirement
name: feature-check --deep shall offer minimal correction sets (how to fix) for void models
status: draft
reqDomain: software
verificationMethod: test
---

A conflict explanation ([[REQ-TRS-FMA-007]]) tells the author *what* is in tension; a **diagnosis** tells them *how to fix it*. For a **void** model (`E223`), the tool **shall** additionally report one or more **minimal correction sets (MCS)**.

### Definition

- A **correction set** is a set of **relaxable** authoring constraints whose removal makes the model satisfiable. Relaxable constraints are the authoring choices: `requires`, `excludes`, `mandatory`, group at-least / at-most, and the root assertion. The structural `child ⇒ parent` implications are **not** offered as fixes (they are not authoring choices).
- A reported MCS **shall** be **sound** (removing exactly that set makes the model satisfiable) and **minimal** (no proper subset is itself a correction set).

### Reporting

- At least one MCS **shall** be reported with the `E223` finding and in `--json` (`diagnoses`: a list of correction sets, each a list of constraint descriptions).
- Diagnoses are complementary to the conflict core: the core is *what conflicts*, an MCS is *what to relax*. (Where feasible the tool **may** report several MCSs.)
- Deterministic ([[REQ-TRS-FMA-006]]); Boolean layer only.

**Source:** ADR-FM-002; complements REQ-TRS-FMA-007 (conflict ↔ correction duality).

**Acceptance criteria:** For a model void due to `A requires B` together with `A excludes B`, the diagnosis reports a correction set each of whose removal restores satisfiability (e.g. `{A excludes B}` and/or `{A requires B}`), and removing a reported MCS makes the model non-void (soundness); no reported MCS has a satisfiable proper subset that is also a correction (minimality). Output is identical across runs.
