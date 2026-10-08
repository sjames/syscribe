---
type: PlanningItem
id: PI-MCP-MEM-001
name: "Measure MCP memory on a 12,000-element model, then reduce it"
status: todo
itemType: feature
achieves: [REQ-TRS-MCP-MEM-000]
tags:
  - mcp
  - memory
---

Ordered by `docs/design/mcp-memory.md`: log and expose resident memory and add a 12,000-element
benchmark with a ceiling; then stop holding two stores across a reload; then validate guarded
writes against an overlay instead of a full candidate copy; then keep bodies on disk. Interning,
slimmer elements and lazy loading follow only if the measurements still call for them.
