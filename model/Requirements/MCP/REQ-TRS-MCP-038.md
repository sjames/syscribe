---
type: Requirement
id: REQ-TRS-MCP-038
name: "evidence returns a requirement's verification chain with verdicts"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPEvidenceToolsADR
tags:
  - mcp
---

The MCP server shall expose an `evidence` read tool that, given a single requirement reference,
returns its verification chain.

## Behaviour

- The result shall list each verifying TestCase → its `testFunctions[].function` / `sourceFile` →
  the latest ingested verdict (and run timestamp when available).
- When no results sidecar is present, verdicts shall be reported as `unknown` (not omitted), so a
  client can distinguish "no test" from "test not yet run".
