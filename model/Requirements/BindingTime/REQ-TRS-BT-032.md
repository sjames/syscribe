---
type: Requirement
id: REQ-TRS-BT-032
name: "A dependent that relies on a retained element being absent is reported"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

A requirement, `satisfies:` link or test that relies on the absence of a variant-retained element shall produce a finding.

**Acceptance criteria:**
- A test that verifies the absence of a retained element is reported; one that does not is silent.
