---
type: Requirement
id: REQ-TRS-BT-015
name: "A required parameter whose window has closed and is still unbound is an error"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

A required parameter with no default whose window closed before the Configuration's stage, and that no Configuration in the chain binds, shall be an error (`E757`, in place of the `W017` warning that parameter would otherwise get); a parameter whose window is still open shall be residual variability ([[REQ-TRS-BT-016]]), not an error.

**Acceptance criteria:**
- A `compile`-window required parameter unbound in a `load`-stage Configuration is an error.
- The same parameter with a `load` window in that Configuration is residual and silent.
