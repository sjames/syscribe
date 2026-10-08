---
type: Requirement
id: REQ-TRS-FMED-002
name: "Dead, core, false-optional and void are marked on the feature diagram, each with the reason, and update live"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-FMED-000]
breakdownAdr: Decisions::FeatureModelEditorADR
tags:
  - feature-model
  - variability
---

A read-only endpoint `GET /api/feature-model/analysis` shall return, from the SAT engine, whether the model is void and which features are dead, core and false-optional, and for each dead or false-optional feature and for a void model the constraints responsible, as human labels. The `/features` page shall mark each state on the diagram and show the reason in the Analysis panel on selection, and shall refresh when the model reloads. A model too large for deep analysis shall say so instead of showing nothing.
