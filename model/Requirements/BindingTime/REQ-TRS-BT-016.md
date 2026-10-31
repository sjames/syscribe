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

`residual --config X [--json]` shall list every feature and parameter not bound by `X`'s effective (inherited plus own) selection, grouped by stage and by the stage's `binder`, with the window still available to each; an item whose window closed before the Configuration's stage shall be listed apart as closed. A feature takes part only when it is a real choice (not abstract or mandatory), a parameter only when its feature is selected and it is neither fixed nor derived; an item with no window is not listed. The MCP server shall expose it read-only.

**Acceptance criteria:**
- On a `compile` Configuration, `residual` lists the `load` and `runtime` items and none of the compile-bound ones.
- An item bound by the Configuration, or fixed, is not listed.
- An unbound item whose window closed before the Configuration's stage is in the closed group.
- `--json` output is stable and sorted, groups earliest stage first.
