---
id: REQ-TRS-MG-017
type: Requirement
name: Tool shall render the allocation and trade-study reports as 2-D matrix grids
status: draft
reqDomain: software
verificationMethod: test
---

The two companion MagicGrid reports — **`matrix --allocations`** and **`trade-study`** —
**shall** offer a true **2-D matrix grid** rendering (axes with a cell per intersection),
so the allocation coverage and the trade scores read as grids rather than flat lists
(Tier 3).

### Allocation matrix

- `matrix --allocations` **shall** render allocation as a **source × target** matrix: one
  row per source element, one column per target element, a **`✓`** in a cell where an
  allocation edge exists and a blank/`·` otherwise. A source row with **no** allocation
  (a gap) **shall** be visibly marked, consistent with the existing gap rollup and the
  `MG081` coverage check.

### Trade-study matrix

- `trade-study` **shall** render a **Configuration × MoE** scored matrix: one row per
  candidate `Configuration`, one column per Measure of Effectiveness, each cell carrying
  the score (or `n/a` for an unevaluable/ambiguous binding, per [[REQ-TRS-MG-012]]), with
  the **winning** configuration highlighted. The weighted total column is retained.

### Consistency and compatibility

- The matrices **shall** be derived from the **same** allocation edge extractor
  ([[REQ-TRS-ALLOC-001]]) and MoE/weight logic the existing reports use — the grid is a
  re-presentation, not a re-computation.
- `--json` output for both commands **shall** be unchanged (additive at most); the grid is
  the human rendering.

**Source:** user request for grid views across the MagicGrid reports (Tier 3). Companion of
[[REQ-TRS-MG-015]] (grid report) and [[REQ-TRS-MG-016]] (SVG); reuses the allocation and
trade-study engines.

**Acceptance criteria:**

- `matrix --allocations` on `model_mg/` prints a source×target matrix with `✓` in
  allocated cells; a source with no allocation is marked as a gap.
- `trade-study` on `model_mg/` prints a Configuration×MoE score matrix with the winning
  configuration highlighted and the weighted total retained.
- Both commands' `--json` output is unchanged (no field removed).
