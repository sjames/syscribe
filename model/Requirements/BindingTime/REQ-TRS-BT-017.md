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

`parameterConstraints:` (§9.7) shall be evaluated on a partial binding as satisfied, violated or undetermined; a constraint violated by the bound subset alone shall be reported even if other parameters are open (`E221`/`W025` under `feature-check`, with the open parameters named), and an undetermined one shall not be an error. An open parameter spans the range it declares, narrowed by `parameterNarrowing:` along the chain (everything, when it declares no range), so a reported violation holds for every value the open parameters can still take. A parameter is a point only when a Configuration bound it, it is fixed, or its `default:` is final (it has no window, or the window closed before the Configuration's stage); a default a later stage can still change does not make it one. This partial evaluation applies to a staged Configuration; an unstaged one is judged only when fully evaluable, as before. `residual` shows each applicable constraint's state.

**Acceptance criteria:**
- `a + b <= 10` with `a = 12` bound and `b` open is reported violated.
- The same constraint with `a = 2` and `b` open is undetermined and silent.
