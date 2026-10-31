---
type: Requirement
id: REQ-TRS-BT-030
name: "params check reports a values file that is stale against the model"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

`params check` shall compare the file header's projection hash with the current model and report a mismatch — or a missing hash header — as a warning (`W751`), gateable with `--deny`, in the manner of `W091`.

**Acceptance criteria:**
- Changing a parameter's `range:` in the model makes a previously generated file report staleness.
