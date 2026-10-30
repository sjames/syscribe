---
type: Requirement
id: REQ-TRS-BT-016
name: "residual lists the variability a Configuration leaves open, by stage and binder"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

`residual --config X [--json]` shall list every feature and parameter not bound by `X`'s effective (inherited plus own) selection, grouped by stage and by the stage's `binder`, with the window still available to each; the MCP server shall expose it read-only.

**Acceptance criteria:**
- On a `compile` Configuration, `residual` lists the `load` and `runtime` items and none of the compile-bound ones.
- `--json` output is stable and sorted.
