---
type: Requirement
id: REQ-TRS-BT-018
name: "configure --stage reports the forced and free choices at a stage"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

`configure --stage S` shall report which choices are forced and which free at stage `S`, using the existing forced/free analysis restricted to items whose window contains `S`; a feature with no window is unrestricted. An unknown stage is a usage error.

**Acceptance criteria:**
- Items whose window excludes `S` are not listed as choices.
- Output agrees with `residual` for the same Configuration.
