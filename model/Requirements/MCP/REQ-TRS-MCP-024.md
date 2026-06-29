---
type: Requirement
id: REQ-TRS-MCP-024
name: "Server advertises the MCP logging capability and honours setLevel"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
---

The MCP server shall advertise the MCP `logging` capability and accept `logging/setLevel`
requests, so a client can receive structured diagnostic messages over the protocol instead of
relying on the server's stderr.

## Behaviour

- The `initialize` result shall declare the `logging` capability.
- A `logging/setLevel` request shall be accepted and succeed.
- The server shall emit at least one structured log notification (`notifications/message`) for a
  significant event (for example, a model reload), at or above the client-set level.
