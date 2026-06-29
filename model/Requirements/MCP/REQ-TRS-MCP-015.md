---
type: Requirement
id: REQ-TRS-MCP-015
name: "check_ref and next_id pre-write guard tools"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - authoring
---

The MCP server shall expose two cheap pre-write guard tools so an LLM can validate references
and allocate identifiers before committing a write.

## Tools

- **`check_ref {ref}`** shall report whether the given reference resolves and, when it does,
  the resolved element's `qname`, `id`, and `type` — letting the client verify a cross-reference
  (`supertype`, `verifies`, `derivedFrom`, …) before writing it.
- **`next_id {prefix}`** shall return the next available stable id for a given prefix (e.g.
  `REQ-TRS-MCP` → the next free numeric suffix), reusing the CLI next-id logic, so created
  elements never collide on id.
