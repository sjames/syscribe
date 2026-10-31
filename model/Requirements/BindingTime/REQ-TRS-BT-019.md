---
type: Requirement
id: REQ-TRS-BT-019
name: "feature-check --deep verifies that a staged chain remains satisfiable"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

`feature-check --deep` shall verify, for a staged `derivedFrom:` chain, that each stage's bindings leave at least one valid completion in the later windows; an unsatisfiable stage shall be an error (`E758`) naming the Configuration that closed the last valid completion — the earliest in the chain, not the ones that merely inherit its choices. A staged Configuration that another staged Configuration is derived from is deliberately partial, so it is judged by this check and not by the completeness rules: `E225` does not apply to it, and `E219` is not raised for a required feature whose window is still open at its stage. A Configuration nothing refines further is judged as before.

**Acceptance criteria:**
- A `compile` Configuration whose bindings leave no valid completion of a later feature is reported, naming it.
- A satisfiable chain is clean, and a partial Configuration whose required feature is bound later raises neither `E219` nor `E225`.
- An unrefined or unstaged Configuration missing a required feature is still `E219`.
