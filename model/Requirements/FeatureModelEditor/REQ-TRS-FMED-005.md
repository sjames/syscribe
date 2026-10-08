---
type: Requirement
id: REQ-TRS-FMED-005
name: "For a feature, the elements it gates, the configurations that select it and the effect of removing it are shown"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-FMED-000]
breakdownAdr: Decisions::FeatureModelEditorADR
tags:
  - feature-model
  - variability
---

`GET /api/feature-model/impact/<feature>` shall return the elements conditioned on the feature by `appliesWhen:` grouped by type with their identifiers, the configurations that select it, and the features that require or exclude it. The page's Impact panel shall show them with links into the model.
