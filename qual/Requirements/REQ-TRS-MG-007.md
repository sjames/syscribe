---
id: REQ-TRS-MG-007
type: Requirement
name: Tool shall render a MoE-weighted trade study comparing configurations
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** render a **trade study** that scores and ranks the model's
`Configuration`s against its Measures of Effectiveness, so the MoEs defined by
[[REQ-TRS-MG-004]] are actually *used* to compare solution alternatives (MagicGrid B4 feeds
the trade study that selects a solution). This is a **read-only reporting** feature like the
Requirement × Configuration `matrix` and the allocation matrix of [[REQ-TRS-MG-006]];
available regardless of profile, MagicGrid-aware in that its rows are the `mg_moe` elements
of [[REQ-TRS-MG-004]].

### Command

- `syscribe -m <root> trade-study [--json] [--config <C> ...]` **shall** build a matrix
  whose **rows** are the MoEs (`custom_fields: { mg_moe: true }`) and whose **columns** are
  the model's `Configuration`s (all of them, or the subset named by `--config`).

### Per-cell evaluation

For each (MoE, Configuration) the tool **shall**:

1. **Evaluate the MoE host element's calculation** to a numeric value under that variant by
   taking the host's `expression` (dropping any `LHS =` assignment prefix) and resolving each
   variable token from the `Configuration`'s `parameterBindings` — a binding matches a token
   by the final `.`/`::`-segment of its key, falling back to an exact-key match. A token that
   resolves to no binding makes the cell unevaluable (see the `n/a` rule below). This
   realises "the MoE value under the variant" through the configuration's bound parameters;
   it does not require a feature model, so the read-only report works on any model that
   parses.
2. **Normalise** the value to a score in `[0, 1]` against the MoE's bounds and direction
   ([[REQ-TRS-MG-004]] `mg_moe_*`):
   - `maximize`: `score = clamp((value − threshold) / (objective − threshold), 0, 1)`
   - `minimize`: `score = clamp((threshold − value) / (threshold − objective), 0, 1)`
   - a value worse than `mg_moe_threshold` yields score `0` **and** is flagged as a
     **threshold violation** (a knock-out, not merely a low score).
3. Report the raw value, the `[0,1]` score, and the weighted contribution
   `mg_moe_weight × score` (weights default to equal when absent; the tool **shall**
   normalise weights so they sum to 1 for the rollup).

### Rollup

- A footer **shall** report, per configuration, the **weighted total score** and a rank,
  marking the top-scoring configuration; any configuration with a threshold violation
  **shall** be marked as failing the trade even if its weighted score is high.
- A cell whose MoE cannot be evaluated under a configuration (host has no numeric result)
  **shall** be shown as `n/a` and excluded from that column's weight normalisation.
- `--json` **shall** emit the full grid (values, scores, weighted contributions) and the
  per-configuration rollup.

**Source:** MagicGrid Measurements of Effectiveness drive a trade study comparing solution
alternatives (B4). Consumes the MoE definitions of [[REQ-TRS-MG-004]] and the §9
configuration projection; parallel to the `matrix` command. Completes the MagicGrid
requirement set [[REQ-TRS-MG-001]]–[[REQ-TRS-MG-006]].

**Acceptance criteria:**

- `trade-study` lists every `mg_moe` element as a row and every `Configuration` as a
  column; each cell shows the projected value, its `[0,1]` score, and the weighted
  contribution.
- A MoE value beyond its `mg_moe_objective` scores `1.0`; a value worse than
  `mg_moe_threshold` scores `0` and is flagged as a threshold violation that fails the
  configuration in the rollup.
- The footer ranks the configurations by weighted total and marks the winner; `--config A
  --config B` restricts the columns to A and B.
- `trade-study --json` emits the grid and rollup as structured JSON; an unevaluable cell is
  reported `n/a` and dropped from that column's weight normalisation.
