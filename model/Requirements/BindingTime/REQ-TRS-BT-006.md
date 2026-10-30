---
type: Requirement
id: REQ-TRS-BT-006
name: "FeatureDef and its parameters declare a binding window with bindingTime and latestBindingTime"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

A `FeatureDef`, and each entry of its `parameters:`, shall accept `bindingTime:` (the earliest stage at which it may be bound) and an optional `latestBindingTime:` (the latest). With only `bindingTime:` the window is that single stage; with neither the item is unspecified and shall be exempt from every binding-time check.

**Acceptance criteria:**
- A feature with `bindingTime: compile` and `latestBindingTime: load` has a window spanning both stages.
- A feature or parameter with neither field produces no binding-time finding of any kind.
