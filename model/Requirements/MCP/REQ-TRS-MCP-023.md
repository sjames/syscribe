---
type: Requirement
id: REQ-TRS-MCP-023
name: "Read-only mode disables the write tools"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - security
---

`syscribe mcp --read-only` shall start the server with all mutating tools disabled, so the
model can be browsed and validated safely in contexts where writes must not be possible.

## Behaviour

- In read-only mode the write tools (`create_element`, `update_element`, `move_element`,
  `delete_element`, `apply_changes`) shall not be advertised in `tools/list` and shall not be
  invocable.
- The full read/query/authoring surface (`search`, `get_element`, …, `coverage`) shall remain
  available unchanged.
- The default (without `--read-only`) shall continue to expose the write tools.
