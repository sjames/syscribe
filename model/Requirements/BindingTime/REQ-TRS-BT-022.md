---
type: Requirement
id: REQ-TRS-BT-022
name: "Provenance is shown by show, configure and the JSON outputs"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

`show <Configuration>`, `configure`, and the `--json` forms of `residual` and `params export` shall include the provenance of [[REQ-TRS-BT-021]].

**Acceptance criteria:**
- Each surface lists provenance for the same value identically.
