---
type: Requirement
id: REQ-TRS-BT-035
name: "matrix offers a per-stage coverage view"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

`matrix --stage S` shall give the Requirement × Configuration coverage grid as it can be known at stage `S`: a feature whose window extends beyond `S` and that no Configuration in the column's `derivedFrom:` chain has decided by `S` is unknown there, so a requirement (or its covering test) that depends on it is shown `open` (`?`), neither covered nor a gap and not counted as applicable. Without `--stage` the grid is as authored.

**Acceptance criteria:**
- `matrix --stage load` differs from `matrix --stage compile` only in cells whose requirement depends on a feature decided between those two stages.
- An unknown stage is a usage error; the JSON names the stage only when one is given.
