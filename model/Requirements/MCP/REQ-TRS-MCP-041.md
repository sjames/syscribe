---
type: Requirement
id: REQ-TRS-MCP-041
name: "diagram_coverage reports view-model drift in both directions"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPEvidenceToolsADR
tags:
  - mcp
---

The MCP server shall expose a `diagram_coverage` read tool reporting drift between the model and its
diagrams in both directions.

## Behaviour

- It shall report (a) in-scope model elements referenced by **no** `Diagram` shape, and (b) diagram
  shapes/edges whose `ref` resolves to no model element (the `W402` set).
- Inputs: `{ root?, types? }` to scope the analysis to a namespace subtree and/or element types.
