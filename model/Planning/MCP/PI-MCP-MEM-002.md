---
type: PlanningItem
id: PI-MCP-MEM-002
name: "MCP memory, second round: validate guarded writes without a full copy; one store at a time on reload; bounded body cache"
status: todo
itemType: feature
achieves: [REQ-TRS-MCP-MEM-000]
tags:
  - mcp
  - memory
---

The ideas left open in `docs/design/mcp-memory.md`: re-check only what a change can affect instead
of walking and validating a second copy of the model, serialise the reload swap so two stores are
never alive together, and keep Markdown bodies behind a bounded cache.
