---
type: Requirement
id: REQ-TRS-MCP-028
name: "Feature-model inspection tools: features and feature_check"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - variability
---

The MCP server shall expose tools to inspect and validate the variability (feature) model.

## Tools

- **`features {feature?}`** — with no argument, return an overview of the feature model: each
  `FeatureDef`'s qname, id, name, group kind, mandatory flag, parent, `requires`/`excludes`, and
  declared parameters; with a `feature` reference, return that single feature's card.
- **`feature_check {deep?}`** — return the holistic feature-model validation findings; when
  `deep` is set, additionally return SAT-backed analysis: whether the model is void, and the
  dead, core, and false-optional features, invalid configurations, and minimal diagnoses.

When the model carries no feature model, both tools shall return a clear dormant response (e.g.
`{hasFeatureModel: false}`) rather than an error.
