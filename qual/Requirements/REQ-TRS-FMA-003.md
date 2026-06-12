---
id: REQ-TRS-FMA-003
type: Requirement
name: feature-check --deep shall detect void models, dead features, false-optional features, and report core features
status: draft
reqDomain: software
verificationMethod: test
---

Over the propositional encoding ([[REQ-TRS-FMA-001]]), `feature-check --deep` **shall** compute the following automated-analysis operations. Each is defined as a decision over the encoded formula `Φ` (let `Φ` be the conjunction of all encoding clauses, including the asserted root).

| Result | Definition | Finding |
|---|---|---|
| **Void model** | `Φ` is **unsatisfiable** — no valid configuration exists | `E223` (error), once, on the feature-model root/package; with an explanation per [[REQ-TRS-FMA-005]] |
| **Dead feature** `F` | `Φ ∧ F` is **unsatisfiable** — `F` is in no valid configuration | `E224` (error), one per dead feature, on `F`'s file |
| **Core feature** `F` | `Φ ∧ ¬F` is **unsatisfiable** — `F` is in every valid configuration | reported (not an error): listed in `--json coreFeatures` and the text summary |
| **False-optional** `F` | `F.groupKind: optional` with parent `P`, and `Φ ∧ P ∧ ¬F` is **unsatisfiable** — `F` is forced whenever `P` is selected | `W018` (warning), on `F`'s file |

Rules:

- **Void dominates.** When the model is void (`E223`), the tool **shall not** additionally emit a dead-feature finding for every feature (all are trivially dead); it reports `E223` plus its explanation and **may** skip the per-feature analyses.
- Dead-feature and false-optional analysis **shall** run only when the model is **non-void**.
- All emitted feature lists/findings **shall** be ordered deterministically (qualified name).
- These analyses concern the **Boolean** layer only; parameter satisfiability is out of scope ([[REQ-TRS-FMA-006]]).

**Source:** ADR-FM-001; automated analysis of feature models (void/dead/core/false-optional).

**Acceptance criteria:** A model with globally conflicting constraints (e.g. a mandatory feature that `excludes` another mandatory feature) yields exactly one `E223` and no per-feature dead spam. A non-void model containing a feature reachable in no configuration yields an `E224` on that feature and not on others. A feature present in every configuration appears in `coreFeatures`. An `optional` feature that some constraint forces on with its parent yields `W018`. A model with none of these conditions yields none of `E223`/`E224`/`W018` and a `coreFeatures` list limited to genuinely-core features.
