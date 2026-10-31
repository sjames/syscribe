---
type: Requirement
id: REQ-TRS-BT-029
name: "params check validates a values file against the model"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

`params check FILE --config X` shall validate a values file against the contract — unknown keys, wrong type, out-of-range value, value not in `enumValues:`, a different value for an `isFixed:` parameter, a missing required parameter — reusing `E204`/`E205`/`E206`, with `E760` (unknown key), `E761` (wrong type) and `E762` (missing or still-open required parameter), and shall exit non-zero on any error-level finding.

**Acceptance criteria:**
- A file edited to give an out-of-range value fails, naming the key.
- A clean generated file passes.
