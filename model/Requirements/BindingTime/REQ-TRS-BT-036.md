---
type: Requirement
id: REQ-TRS-BT-036
name: "The feature-model viewer shows windows and stages and filters by stage"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

`/features` shall show each feature's and parameter's binding window and each Configuration's `bindingStage:`, and shall filter by a chosen stage, highlighting residual variability.

**Acceptance criteria:**
- The stage filter's residual set agrees with `residual --json` for the same Configuration.
