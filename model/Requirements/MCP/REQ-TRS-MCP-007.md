---
type: Requirement
id: REQ-TRS-MCP-007
name: "move_element tool renames/relocates and rewrites references"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - write
---

The MCP server shall expose a `move_element` write tool that renames or relocates an existing
element to a new qualified name and rewrites all cross-references to it across the model,
reusing the CLI `move` implementation.

## Behaviour

- The destination qualified name shall be validated before any change is made.
- All inbound references to the moved element (e.g. `supertype`, `verifies`, `derivedFrom`,
  `satisfies`) shall be updated to the new name, and the tool shall report which files were
  rewritten.
- The tool shall honour the common write-guard behaviour of REQ-TRS-MCP-008.
