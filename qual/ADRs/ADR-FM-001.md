---
type: ADR
id: ADR-FM-001
title: "Solver-backed feature-model analysis: in-tree SAT over the boolean layer, numeric/SMT deferred"
status: accepted
---

## Context

The shipped `feature-check` command (REQ-TRS-FM-001…003) is **extensional**: it validates the
`Configuration`s that have actually been authored, plus light structural checks (`requires`/`excludes`
resolution and satisfaction over authored configs, dead/always-on optional by selection *count*) and a
DFS `derivedFrom:` cycle check. It performs **no symbolic reasoning over the feature model's
configuration space.**

This cannot answer the questions that automated analysis of feature models (AAFM) exists to answer —
the questions that matter for a safety-critical product line:

- **Void model** — does *any* valid configuration exist at all?
- **Dead feature** — a feature that can appear in *no* valid configuration (not merely "absent from the
  configs someone wrote"). The shipped `W011` is a count over authored configs and does **not** detect
  logically dead features (e.g. `A requires B` ∧ `A excludes B`).
- **False-optional feature** — declared `optional` but forced selected whenever its parent is.
- **Core features** — present in every valid configuration.
- **Configuration validity under full semantics** — does an authored selection actually satisfy the
  group/cardinality/cross-tree rules, not just `requires`/`excludes`?

These are decision problems over the propositional encoding of the feature model — i.e. SAT/BDD
territory. Feature *parameters* (numeric `range:`, `derivedFrom:` arithmetic, `parameterConstraints`
expressions) push beyond Boolean SAT into SMT/CSP, and the format spec deliberately leaves the
`parameterConstraints` expression language opaque (§9.7).

The tool is qualification-targeted (ISO 26262-8 §11 / IEC 61508-3): results must be **deterministic**
and builds **reproducible**, which constrains the choice of decision procedure and its dependencies.

## Decision

1. **Keep the extensional `feature-check` as the default** — it is cheap, always-on, and catches real
   authoring mistakes. It is unchanged.

2. **Add solver-backed deep analysis behind `feature-check --deep`** that encodes the *Boolean* layer
   of the feature model (REQ-TRS-FMA-001) and computes void / dead / core / false-optional and
   full-semantics configuration validity (REQ-TRS-FMA-002…005).

3. **Engine: an in-tree, dependency-free pure-Rust DPLL SAT solver** (`syscribe-model::sat`). This was
   chosen over an external BDD crate during implementation for three reasons: (a) it keeps the build
   **fully dependency-free and offline-reproducible**, which is the strongest qualification posture;
   (b) deletion-based **unsat cores fall out directly**, giving the FMA-005 explanations with no extra
   machinery; (c) determinism is guaranteed by construction (ascending variable order, `true`-before-
   `false` branching). Each analysis is a satisfiability query under assumptions (void = `Φ` UNSAT;
   dead `F` = `Φ ∧ F` UNSAT; core `F` = `Φ ∧ ¬F` UNSAT; false-optional, config-validity likewise). A
   reduced-ordered BDD (`biodivine-lib-bdd`) remains a documented future option if whole-space metrics
   (configuration counting, large-model performance) ever need it.

4. **Boolean layer only.** Numeric/parameter constraints and `parameterConstraints` expression
   *evaluation* (the `E221` class) are **out of scope** for this work and deferred to a future SMT/CSP
   ADR; they require an expression engine first (REQ-TRS-FMA-006).

## Alternatives considered

- **A — Stay extensional only.** Rejected: cannot detect void / true-dead / false-optional / core, which
  are the entire point of feature-model analysis for a product line.
- **B — SAT solver (chosen, implemented in-tree).** void/dead/core/false-optional/config-validity are
  all UNSAT checks under assumptions; determinism is enforced by construction (fixed variable order,
  fixed branch order); unsat cores give explanations directly. An *in-tree* DPLL solver was preferred
  over an external SAT crate (`varisat`/`splr`/`batsat`) to keep the dependency surface zero for
  qualification. Drawback: whole-space *counting* would need many calls — not currently required.
- **C — BDD.** Canonical and good for configuration *counting*; rejected for now because it adds an
  external crate and does not yield unsat cores naturally. Kept as a documented future option behind the
  size guard if counting/large-model performance is needed (REQ-TRS-FMA-006).
- **D — SMT (z3 or similar).** Necessary for the numeric/parameter layer, but a heavy native dependency
  that undermines reproducible qualification builds. Deferred to a later phase.
- **E — External solver process** (emit DIMACS, shell out to MiniSat/Kissat). Rejected: an external
  binary breaks build/runtime reproducibility and the qualification dependency story.

## Consequences

- **+** Real anomaly detection over the *entire* configuration space; deterministic and qualifiable.
- **+** No external process and no native/system dependency — pure-Rust, in-process.
- **+** The `--deep` flag keeps the expensive analysis opt-in; default `feature-check` stays fast.
- **−** SAT is worst-case exponential; the per-feature dead/core analysis issues O(features) solver
  calls and core minimisation O(constraints) more. Mitigated by the feature-count size guard with a
  graceful skip (REQ-TRS-FMA-006); a BDD remains the escalation path.
- **−** Numeric/parameter constraints remain unanalyzed until the deferred SMT phase; users must not read
  "feature model OK" as covering parameter satisfiability.
- **−** Potentially two engines (BDD + SAT fallback) to maintain.
