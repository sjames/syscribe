---
type: Requirement
id: REQ-TRS-MCP-032
name: "why_active tool explains an element's activation under a configuration"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - variability
---

The MCP server shall expose a `why_active {ref, config}` tool that explains whether a given
element is active under a given configuration (or feature selection) and why.

## Behaviour

- Returns `{active: bool, effectiveAppliesWhen?, source, referencedFeatures: [...], verdict}`
  where `source` indicates whether the gating `appliesWhen` is the element's own or inherited
  from an ancestor package, and `verdict` is `active` / `inactive` / `always-active`.
- The explanation shall use the element's **effective** `appliesWhen` (own or inherited),
  canonicalising feature ids to qnames.
- Returns a clear dormant response when no feature model is present.
