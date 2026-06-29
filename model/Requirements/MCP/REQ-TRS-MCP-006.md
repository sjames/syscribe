---
type: Requirement
id: REQ-TRS-MCP-006
name: "update_element tool patches frontmatter and body non-destructively"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - write
---

The MCP server shall expose an `update_element` write tool that patches the frontmatter fields
and/or documentation body of an existing element identified by id, qualified name, or display
name.

## Non-destructive round-trip

- The tool shall mutate only the requested frontmatter keys (setting or clearing them) and
  shall preserve all other frontmatter keys — including keys the schema does not recognise —
  and the Markdown body, by reusing the established frontmatter read–parse–re-emit round-trip.
- When only frontmatter is patched, the body shall be left byte-for-byte unchanged, and vice
  versa.
- The tool shall honour the common write-guard behaviour of REQ-TRS-MCP-008.
