---
type: Requirement
id: REQ-TRS-MCP-025
name: "Server notifies clients when the model changes"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
---

The MCP server shall advertise resource-change capabilities and notify connected clients when
the model it serves changes, so a long-lived client can keep its view in sync.

## Behaviour

- The `initialize` result shall declare the `resources` capability with `listChanged` (and
  `subscribe`) support.
- When the model changes as a result of a committed write tool (REQ-TRS-MCP-005..008,
  020, 021) or an explicit `reload` (REQ-TRS-MCP-002), the server shall emit a
  `notifications/resources/list_changed` notification.
- The notification mechanism is driven by the server's own model mutations and `reload`; a
  filesystem watcher for edits made entirely outside the server is **not** required (the
  `reload` tool covers that case).
