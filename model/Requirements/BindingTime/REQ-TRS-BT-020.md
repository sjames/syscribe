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

`validate --config X` shall treat an item whose window lies after `X`'s stage as open, so a required parameter whose window extends beyond that stage — one a later stage can still bind — does not raise `W017` — under `validate --config` and for every staged Configuration under whole-model `validate`.

**Acceptance criteria:**
- A required `runtime`-window parameter unbound in a `compile` Configuration raises no `W017`.
- A required `compile..load` parameter is not `W017` at a `compile` Configuration (`load` can still bind it), and a required `compile`-only one still is.
- The same parameter with a `compile` window still does.
