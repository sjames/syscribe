---
type: Requirement
id: REQ-TRS-BT-020
name: "validate --config at a stage treats later-stage items as open, not unbound"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

`validate --config X` shall treat an item whose window lies after `X`'s stage as open, so a required later-stage parameter does not raise `W017`.

**Acceptance criteria:**
- A required `runtime`-window parameter unbound in a `compile` Configuration raises no `W017`.
- The same parameter with a `compile` window still does.
