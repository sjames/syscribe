---
type: Requirement
id: REQ-TRS-BT-014
name: "Values bound by an ancestor are closed and may only be narrowed"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

A selection or value bound by a `derivedFrom:` ancestor shall be closed: a descendant shall not rebind it, and may narrow a `range:` or `enumValues:` but shall not widen it; each violation is an error.

**Acceptance criteria:**
- A child that rebinds a parameter its base bound is an error.
- A child that narrows the range is clean; one that widens it is an error.
