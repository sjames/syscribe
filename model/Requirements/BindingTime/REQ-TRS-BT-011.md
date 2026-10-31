---
type: Requirement
id: REQ-TRS-BT-011
name: "A Configuration declares the stage it binds at with bindingStage"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

`Configuration` shall accept `bindingStage:` naming a stage of the effective vocabulary. A Configuration without it is unstaged and shall be checked exactly as today; an unknown stage name is an error (`E751`) and the Configuration is then checked as unstaged.

**Acceptance criteria:**
- `bindingStage: load` on a Configuration validates; `bindingStage: bogus` is an error.
- An unstaged Configuration produces the same findings as before.
