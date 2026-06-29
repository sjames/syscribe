---
type: Requirement
id: REQ-TRS-MCP-012
name: "describe_type tool returns an element type's frontmatter schema"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - authoring
---

The MCP server shall expose a `describe_type` tool that, given an element type name, returns
that type's frontmatter schema as structured data, so an LLM can author a valid element
without parsing the prose specification.

## Returned schema

- For each frontmatter field the tool shall report its name, whether it is required, its value
  type, and — for enumerated fields (e.g. `status`, `reqClass`, `reqDomain`, `testLevel`,
  safety/security levels) — the permitted values.
- The tool shall also be able to enumerate the recognised element types, so a client can
  discover what to describe.
- Results shall be structured JSON, not prose, so the schema is directly actionable.
