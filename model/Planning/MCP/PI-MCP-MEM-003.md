---
type: PlanningItem
id: PI-MCP-MEM-003
name: "MCP memory, third round: the watcher's reload without a second model, and Markdown bodies behind a bounded cache"
status: todo
itemType: feature
achieves: [REQ-TRS-MCP-MEM-000]
tags:
  - mcp
  - memory
---

What `docs/design/mcp-memory.md` leaves open, with its reasons: a pre-scan for torn files so the watcher can
drop the live model before it loads the replacement, and an access API for `RawElement::doc` that a bounded
cache can sit behind.
