---
type: Requirement
id: REQ-TRS-COVROLL-001
name: "matrix --rollup lists every requirement with its own and below-it coverage and a per-class footer"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - coverage
---

`matrix --rollup` shall show, for every requirement at once, what `coverage tree` shows per root (GH #252, `matrix --rollup`).

## Behavior

- One row per native requirement (filterable with `--tag`, `--status`; `--config` projects the model first): `own` (a leaf: `verified`/`planned`/`uncovered`; a parent: the number of direct active tests), `below` (`<active>/<total> active, <n> planned` over the distinct leaves beneath it, `-` for a leaf), the verdict glyph and the `[coverage]` rule applied (`coverage tree` semantics, including the integrity guard — a configuration error exits 1).
- A footer per `reqClass` counts the verdicts (`●` complete, `◐` partial, `○` none, `·` n/a).
- `--json` carries `rows[{id, name, reqClass, status, leaf, own, directTests, leavesActive, leavesPlanned, leavesUncovered, verdict, glyph, rule}]` and `byClass{class: {complete, partial, none, na}}`.
