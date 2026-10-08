---
type: Requirement
id: REQ-TRS-FMED-006
name: "Two configurations are compared and features are shown against configurations in a matrix"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-FMED-000]
breakdownAdr: Decisions::FeatureModelEditorADR
tags:
  - feature-model
  - variability
---

The page shall show the difference between two configurations (features selected in one only) and a matrix of features against configurations, both from the stored configurations' selections in the engine's canonical (qualified-name) form, a feature a configuration does not mention counting as off, with a filter by name, id or qualified name that keeps the matches and their ancestors.
