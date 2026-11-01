---
type: Requirement
id: REQ-TRS-BT-004
name: "One BindingTime type owns parsing, ordering and comparison"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

The tool shall implement the vocabulary once, as a single `BindingTime` type in `syscribe-model`, used by the validator, `feature_model`, placeholders, `discover` and the feature-model UI payloads; no code shall compare a binding time against a literal such as `"runtime"` or against a rank number.

**Acceptance criteria:**
- An `ast-grep` search of the workspace finds no remaining literal `"runtime"` binding-time comparison and no `Some(2)` rank test outside the `BindingTime` type.
- The existing `TC-TRS-PARAM-004` passes unchanged.
