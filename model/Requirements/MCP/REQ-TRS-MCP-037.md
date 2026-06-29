---
type: Requirement
id: REQ-TRS-MCP-037
name: "coverage_gaps returns the actionable coverage subset"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPEvidenceToolsADR
tags:
  - mcp
---

The MCP server shall expose a `coverage_gaps` read tool returning only the actionable subset of the
coverage matrix, complementing (not replacing) the existing `coverage` tool.

## Behaviour

- Each returned row shall be classified as: **uncovered** (requirement active in a configuration
  with no verifying TestCase); **failing** (covered but the latest verdict is fail/skip/missing);
  or **unverified-claim** (non-draft requirement with an integrity/`wcet:` obligation but no passing
  measuring TestCase).
- Each row shall carry the requirement ref, the configuration(s) in which the gap holds, the gap
  class, and the governing finding code where one applies (`W010`, `W015`, `W029`).
- Inputs shall support `{ config?, status?, class? }`.
