---
type: Requirement
id: REQ-TRS-FMED-001
name: "A feature model is shown as a feature diagram in FODA notation, laid out automatically, with collapse, search, cross-tree constraints and parameters"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-FMED-000]
breakdownAdr: Decisions::FeatureModelEditorADR
tags:
  - feature-model
  - variability
---

`syscribe-model` shall derive a diagram of kind `FeatureModel` from a `FeatureDef` subtree or a `FeatureModel` sheet, and the browser shall show it at `/features`. Each feature is a node showing its name, id, a mandatory or optional mark, abstract state, and parameters; a feature's children are joined to it by tree edges with an XOR arc for `groupKind: alternative` and a filled OR arc for `groupKind: or`; `requires` and `excludes` constraints are drawn as distinct dashed edges. Layout is automatic. A subtree can be collapsed and expanded, a feature found by name or id with a match highlighted and revealed, and the whole model fitted to the window. The diagram exports as SVG, PNG, PlantUML and Mermaid from the browser and from `diagram export`.
