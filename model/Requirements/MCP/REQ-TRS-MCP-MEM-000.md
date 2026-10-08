---
type: Requirement
id: REQ-TRS-MCP-MEM-000
name: "The MCP server keeps its memory use bounded and observable on models of at least 12,000 elements"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - mcp
  - memory
  - performance
---

The MCP server shall serve a model of at least 12,000 elements without being terminated for memory
use on a host with a stated, documented memory budget, through load, reload and guarded writes. It
shall report its own resident memory at each of those points, so that growth is visible before a
host intervenes. The budget and the measured peak for a reference 12,000-element model shall be
documented and checked in continuous integration.

## Rationale

A server restarted mid-session loses the agent's context of the model and any work in flight. A
user reported a restart on a model of about 12,000 elements. The ideas under consideration are in
`docs/design/mcp-memory.md`.
