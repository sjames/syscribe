---
type: Requirement
id: REQ-TRS-MCP-022
name: "Write tools include a unified text diff of every changed file"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - write
---

Every MCP write tool (`create_element`, `update_element`, `move_element`, `delete_element`, and
`apply_changes`) shall include, in its result, a unified text diff of each file it would change,
so a client can review the exact byte-level change before or after committing.

## Diff content

- For a `dry_run` the diff shall describe the would-be change; for a commit it shall describe
  the change that was applied.
- The diff shall identify each affected file path and show added/removed lines in unified-diff
  form (a creation shows the new file's content as additions; a deletion shows its content as
  removals).
- When a tool changes no file content, the diff shall be empty or omitted.
