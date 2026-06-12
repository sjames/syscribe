---
id: REQ-TRS-FMA-009
type: Requirement
name: feature-check shall count and optionally enumerate the valid configurations a feature model permits
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** report the size of the configuration space the feature model permits, so authored coverage can be put in context (e.g. *"8 authored configurations out of N valid variants"*).

### Behaviour

- **`feature-check --count`** **shall** report the number of valid configurations (distinct satisfying assignments over the feature variables of the encoding `Φ`), computed by **enumeration with blocking clauses**: solve, record the model, add a clause forbidding it, repeat until UNSAT.
- **`feature-check --enumerate`** **shall** additionally list each valid configuration as its set of selected feature qualified names.
- A documented **guard** **shall** cap the work: above a configuration-count cap the tool **shall** stop and report `≥ <cap> (truncated)` rather than enumerate unbounded; the per-model feature-count guard ([[REQ-TRS-FMA-006]]) also applies.
- **`--json`**: `variantCount` (an integer, or `{ "atLeast": <cap> }` when truncated) and, for `--enumerate`, `variants` (a list of feature-name lists). Enumeration order **shall** be deterministic ([[REQ-TRS-FMA-006]]).
- Dormant when no feature model is present (notice, exit `0`); Boolean layer only; the count concerns feature selection, not parameter values.

**Source:** ADR-FM-002.

**Acceptance criteria:** A model whose only variability is one `alternative` group of `n` children under a mandatory parent reports `variantCount` = `n`; adding one independent `optional` feature doubles it to `2n`. `--enumerate` lists exactly those configurations in a stable order. A model whose space exceeds the cap reports a truncated `atLeast` count without hanging. With no `FeatureDef`, it prints a notice and exits `0`.
