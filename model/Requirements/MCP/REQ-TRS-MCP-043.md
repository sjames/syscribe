---
type: Requirement
id: REQ-TRS-MCP-043
name: "MCP tool input schemas are valid for strict (zod) MCP clients"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
---

Every MCP tool's `inputSchema` shall be valid for strict MCP clients, so the whole `tools/list`
loads rather than being rejected wholesale.

## Behaviour

- Each declared input property shall have an **object** JSON Schema; no property shall be a bare
  boolean schema (`true`/`false`). A free-form object argument (e.g. a frontmatter `fields` map)
  shall be declared with a `{"type":"object"}` schema rather than the catch-all that
  `serde_json::Value` would otherwise emit.
- Rationale: zod-based clients (e.g. Claude Code) reject a boolean property schema with
  "Invalid input", which fails the entire tool list — the server then appears connected but
  exposes no tools.
