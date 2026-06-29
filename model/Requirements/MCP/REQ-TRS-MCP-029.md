---
type: Requirement
id: REQ-TRS-MCP-029
name: "configure tool reports completability and forced/free features"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - variability
---

The MCP server shall expose a `configure {config}` tool that, given a `Configuration` (or an
ad-hoc comma-separated feature selection), treats its `features:` as a partial assignment and
reports — via the SAT-backed feature-model engine — whether the selection is completable and
which features are forced-true, forced-false, and free.

## Behaviour

- Returns `{satisfiable, forcedTrue: [...], forcedFalse: [...], free: [...], explanation?}`.
- Returns a clear dormant response when no feature model is present and a clear not-found
  response when the configuration does not resolve.
