---
type: Requirement
id: REQ-TRS-FMED-000
name: "The browser provides a feature model viewer and editor to the standard of the leading product-line tools: see, understand, analyse, configure and change a product line's variability"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - feature-model
  - variability
---

Syscribe shall let a user view a product line's feature model as a diagram, see which features are dead, core, false-optional or contradictory and why, configure a product with live propagation and explanation, and change the feature model with a preview of the effect of each change on its validity, all in the browser and live, using the same engine the command line and MCP server use.

## Rationale

The engine already proves the properties of a feature model; users still reach for other tools to see and change it. Closing that gap is the point of this requirement.
