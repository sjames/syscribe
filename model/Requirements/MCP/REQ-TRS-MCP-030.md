---
type: Requirement
id: REQ-TRS-MCP-030
name: "project tool returns a variant's active elements and projected validation"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - variability
---

The MCP server shall expose a `project {config}` tool that projects the model onto a variant —
a `Configuration` or ad-hoc feature selection — and returns the resolved selection, the set of
active elements, and the findings of validating that projected variant.

## Behaviour

- Returns `{selection: {qname: bool}, active: [{qname, id, type}], activeCount, findings: [...]}`.
- The `findings` shall come from validating the projected subset (escaping-reference checks plus
  standard validation), so an LLM can confirm a specific product variant is well-formed.
- Returns a clear dormant response when no feature model is present.
