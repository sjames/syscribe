---
type: Requirement
id: REQ-TRS-BT-010
name: "Models using the default triad and single-stage bindingTime behave exactly as before"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

For a model that uses the default vocabulary and single-stage `bindingTime:` only, `E229`, `E230`, `W027`, `W017`, `W246` and `W248` shall behave as specified in [[REQ-TRS-PARAM-004]] and [[REQ-TRS-PHOLD-001]].

**Acceptance criteria:**
- `TC-TRS-PARAM-004` and the placeholder and machine-parameter test suites pass unchanged.
