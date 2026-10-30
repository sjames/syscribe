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

`matrix --stage S` shall give the Requirement × Configuration coverage grid at stage `S`.

**Acceptance criteria:**
- `matrix --stage load` differs from `matrix --stage compile` only for items whose window spans both stages.
