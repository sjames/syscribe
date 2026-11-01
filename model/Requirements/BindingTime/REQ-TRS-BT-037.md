---
type: Requirement
id: REQ-TRS-BT-037
name: "The feature-model editor reads and writes the binding-time fields without losing others"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

The editor shall read and write `bindingTime:`, `latestBindingTime:` and `bindingStage:` through the guarded-write engine, preserving every key the form does not show, and shall offer the effective vocabulary of [[REQ-TRS-BT-001]] in its stage pickers.

**Acceptance criteria:**
- Setting a window in the editor round-trips to the file with unrelated keys intact.
