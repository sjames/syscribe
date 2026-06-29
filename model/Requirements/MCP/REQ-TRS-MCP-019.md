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

The tool shall reflect the project's two-level verification model: **leaf** requirements are
verified by a TestCase (typically unit-level), while **parent** requirements (those with
`derivedChildren`) must additionally be verified by an **integration-level** TestCase
(`testLevel` L3, L4, or L5), mirroring validation rule `W305`.

- It shall report the count of requirements that are adequately verified.
- It shall **partition** the verification gaps into two groups, so a client targets real gaps:
  - **`unverifiedLeaves`** — requirements with no `derivedChildren` and no verifying TestCase.
  - **`parentsMissingIntegrationTest`** — requirements with `derivedChildren` that have no
    verifying TestCase at integration level (L3/L4/L5); a parent with only unit-level
    (L1/L2) verifying TestCases still appears here. Each entry shall carry its child count.
- Each listed requirement shall carry its `qname` and `id` so the client can chain follow-up
  calls (e.g. `trace`, `add-testcase-for`).
- It shall optionally surface orphan requirements (no `derivedFrom` and no `derivedChildren`).
