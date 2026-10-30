---
type: Requirement
id: REQ-TRS-BT-003
name: "A malformed [bindingTimes] entry is a warning and is ignored whole"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

A `[bindingTimes]` entry with a cyclic or unsatisfiable order, a duplicate name, an unknown neighbour, or an unknown key shall raise a warning (code allocated at implementation) and shall be ignored whole; the remainder of the vocabulary shall stay in force, as `[linkTypes]` does with `W630`.

**Acceptance criteria:**
- A cyclic `after:`/`before:` pair is reported once and leaves the effective order unchanged.
- An unknown key is reported and the entry is ignored.
