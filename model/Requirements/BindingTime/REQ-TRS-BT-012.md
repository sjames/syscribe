---
type: Requirement
id: REQ-TRS-BT-012
name: "A staged Configuration binds only variability whose window contains its stage"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

A staged Configuration shall select a feature or bind a parameter only if the item's window contains the Configuration's stage; otherwise it shall be an error. For an unstaged Configuration, `W027` (binding a `runtime` parameter) shall be retained as in [[REQ-TRS-PARAM-004]].

**Acceptance criteria:**
- A `compile`-stage Configuration binding a parameter whose window starts at `load` is an error.
- An unstaged Configuration binding a `runtime` parameter still produces `W027`.
