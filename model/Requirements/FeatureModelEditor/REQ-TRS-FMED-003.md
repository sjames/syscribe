---
type: Requirement
id: REQ-TRS-FMED-003
name: "A configurator propagates selections, refuses or explains conflicts, shows the number of valid products, and saves a Configuration"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-FMED-000]
breakdownAdr: Decisions::FeatureModelEditorADR
tags:
  - feature-model
  - variability
---

`POST /api/feature-model/configure` shall take a partial selection and return, for every feature, whether it is selected, deselected, forced on, forced off or free, whether the selection is satisfiable, and for an unsatisfiable one the constraints in conflict; and the number of valid products when it can be computed within a budget, else a lower bound. The page shall let a user select and deselect features, show the propagated state on the diagram at once, mark a conflicting choice with its explanation, and save the result as a `Configuration` through a guarded write, or load an existing one as the starting point.
