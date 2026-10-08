---
type: Requirement
id: REQ-TRS-FMED-004
name: "The feature model is edited in the browser through semantic operations whose effect on validity is previewed before they are written"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-FMED-000]
breakdownAdr: Decisions::FeatureModelEditorADR
tags:
  - feature-model
  - variability
---

`POST /api/feature-model/edit` shall accept an operation (`add`, `remove`, `rename`, `setGroup`, `setMandatory`, `move`, `addConstraint`, `removeConstraint`, `setParameter`) on a feature, apply it to whichever layout the feature lives in, and go through the guarded-write engine. With `preview: true` it shall return, without writing, the features that become dead, false-optional or contradictory, whether the model becomes void, and the configurations that become invalid. The page shall offer each operation on the diagram, show the preview before committing a change that worsens validity, and support undo and redo.
