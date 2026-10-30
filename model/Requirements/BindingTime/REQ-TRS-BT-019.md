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

`feature-check --deep` shall verify, for a staged `derivedFrom:` chain, that each stage's bindings leave at least one valid completion in the later windows; an unsatisfiable stage shall be an error naming the Configuration that closed the last valid completion.

**Acceptance criteria:**
- A `compile` Configuration whose bindings leave no valid completion of a later feature is reported, naming it.
- A satisfiable chain is clean.
