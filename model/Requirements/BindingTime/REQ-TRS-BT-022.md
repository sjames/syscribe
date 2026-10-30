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

`show <Configuration>`, `configure`, and the `--json` forms of `residual` and `params export` shall include the provenance of [[REQ-TRS-BT-021]]. In this phase the surfaces that exist are `show` (a Configuration's binding stage and a parameter-value table) and `configure` (the same table in text, and a `provenance` object in `--json`); `residual` and `params export` carry the same object when they are introduced.

**Acceptance criteria:**
- Each surface lists provenance for the same value identically.
