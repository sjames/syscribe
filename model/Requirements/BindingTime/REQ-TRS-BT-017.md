---
type: Requirement
id: REQ-TRS-BT-017
name: "parameterConstraints are evaluated three-valued on a partial binding"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

`parameterConstraints:` (§9.7) shall be evaluated on a partial binding as satisfied, violated or undetermined; a constraint violated by the bound subset alone shall be reported even if other parameters are open, and an undetermined one shall not be an error.

**Acceptance criteria:**
- `a + b <= 10` with `a = 12` bound and `b` open is reported violated.
- The same constraint with `a = 2` and `b` open is undetermined and silent.
