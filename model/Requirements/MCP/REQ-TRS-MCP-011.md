---
type: Requirement
id: REQ-TRS-MCP-011
name: "Tool failures return structured errors without terminating the server"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
---

A tool call that cannot be satisfied — an unresolved element reference, an unknown tool name,
or invalid/malformed arguments — shall return a structured error result to the client rather
than crashing the process or corrupting the protocol stream, and the server shall remain able
to serve subsequent requests.

## Error behaviour

- A tool invoked with an element reference that resolves to nothing shall return a tool error
  result (the MCP error indicator set, with a human-readable message), not a panic and not a
  silently empty success.
- A malformed request or unknown tool name shall be answered with a well-formed JSON-RPC error
  for that request id; the server shall not exit and shall not write partial or non-protocol
  bytes onto stdout.
- After returning any such error, a subsequent valid request on the same connection shall be
  served normally, demonstrating the server has not been left in a broken state.
