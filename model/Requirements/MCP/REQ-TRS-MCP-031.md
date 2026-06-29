---
type: Requirement
id: REQ-TRS-MCP-031
name: "diff_configs tool compares two variants"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - variability
---

The MCP server shall expose a `diff_configs {a, b}` tool that compares two variants and reports
which elements are active only in A, only in B, so an LLM can see exactly what differs between
two products.

## Behaviour

- Each of `a` and `b` resolves like the `project` tool's `config` argument (a `Configuration`
  or ad-hoc feature selection).
- Returns `{onlyInA: [{qname, id}], onlyInB: [{qname, id}], commonCount}`.
- Returns a clear dormant response when no feature model is present.
