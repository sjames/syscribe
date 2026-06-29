---
type: Requirement
id: REQ-TRS-MCP-020
name: "delete_element tool removes an element under a reference-impact guard"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - write
---

The MCP server shall expose a `delete_element` write tool that removes an existing element,
subject to the common write-guard protocol (REQ-TRS-MCP-008) and confinement
(REQ-TRS-MCP-009), plus a reference-impact guard.

## Reference-impact guard

- The deletion shall be refused (`written: false`) if any other element holds a cross-reference
  to the target, and the tool shall report the blocking inbound references, so the caller can
  resolve them first.
- A caller may override with an explicit `force` (or `cascade`) flag; without it, deleting a
  referenced element is refused.
- `dry_run` defaults to true: a dry-run reports what would be deleted and the blocking
  references without modifying disk. After a successful commit the store is rebuilt.
