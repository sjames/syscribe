---
type: Requirement
id: REQ-TRS-BT-007
name: "An unknown stage or a window that closes before it opens is an error"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

A `bindingTime:` or `latestBindingTime:` naming a stage not in the effective vocabulary, or a `latestBindingTime:` earlier than `bindingTime:`, shall be an error on the declaring element; this generalises `E230` of [[REQ-TRS-PARAM-004]].

**Acceptance criteria:**
- `bindingTime: bogus` is an error; `compile`/`load`/`runtime` are clean.
- `bindingTime: load` with `latestBindingTime: compile` is an error.
