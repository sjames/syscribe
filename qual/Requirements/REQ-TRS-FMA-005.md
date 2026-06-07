---
id: REQ-TRS-FMA-005
type: Requirement
name: Explanations for Void Models and Dead Features
title: feature-check --deep shall explain unsatisfiability with a sound conflict set
status: draft
reqDomain: software
verificationMethod: test
---

An unadorned "model is void" or "feature X is dead" verdict is hard to act on. When the deep analysis reports unsatisfiability, the tool **shall** accompany it with a **diagnostic explanation** so the author can locate the conflict.

### Requirement

- For an `E223` (void model) and for each `E224` (dead feature), the tool **shall** emit an explanation identifying a **set of model constraints** whose conjunction is responsible for the unsatisfiability — an unsat core / conflict set — expressed in terms of `FeatureDef` names and constraint kinds (`requires`, `excludes`, `mandatory`, group/cardinality, root).
- The explanation **shall** be **sound**: every constraint cited is genuinely part of a real conflict (the cited set, together with the relevant assertion, is unsatisfiable). It **need not** be globally minimal, but the tool **should** reduce it toward a minimal core (e.g. by deletion-based minimisation) on a best-effort basis.
- The explanation **shall** be attached to (or printed adjacent to) its finding and included in the `--json` output for that finding.
- Explanations **shall** be deterministic for a fixed model ([[REQ-TRS-FMA-006]]).

### Rationale

This is the difference between "the matrix/feature model is wrong and I don't know why" and an actionable pointer to the two or three constraints in tension — the same philosophy as the W016 no-silent-ignore rule.

**Source:** ADR-FM-001; unsat-core / corrective-explanation analysis of feature models.

**Acceptance criteria:** A model made void by `A requires B` together with `A excludes B` (with `A` forced selected) produces an `E223` whose explanation names `A`, `B`, the `requires` edge and the `excludes` edge; removing either cited constraint makes the model non-void (demonstrating soundness). A dead feature caused by a single `excludes` from a core feature produces an `E224` explanation naming that `excludes` and the two features. The explanation text is identical across two runs.
