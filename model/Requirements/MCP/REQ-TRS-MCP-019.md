---
type: Requirement
id: REQ-TRS-MCP-019
name: "coverage tool summarises requirement verification coverage"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
---

The MCP server shall expose a `coverage` tool that returns a verification-coverage summary for
the model, reusing the model's coverage computation, so an LLM can drive verification
gap-filling.

## Returned summary

- The tool shall report, at minimum, the count of requirements that have at least one verifying
  TestCase and the count and list of requirements that have none.
- It shall optionally surface orphan requirements (no `derivedFrom` and no `derivedChildren`).
- Each listed requirement shall carry its `qname` and `id` so the client can chain follow-up
  calls (e.g. `trace`, `add-testcase-for`).
