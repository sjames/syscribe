---
type: Requirement
id: REQ-TRS-BT-033
name: "Variant-retained elements are listed and unstaged projection is unchanged"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

Variant-retained elements shall be listed by `validate --config X` (as `I750`, informational) and by `residual`, from the one retention report, and projection of an unstaged Configuration, or of a model with no binding windows, shall be byte-identical to the current output.

**Acceptance criteria:**
- The retained list is the same in `validate --config` and `residual`.
- Projecting `model/` and the existing examples produces unchanged output.
