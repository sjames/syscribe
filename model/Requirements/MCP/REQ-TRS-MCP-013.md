---
type: Requirement
id: REQ-TRS-MCP-013
name: "template tool returns a ready-to-edit frontmatter skeleton for a type"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - authoring
---

The MCP server shall expose a `template` tool that returns a ready-to-edit `.md` frontmatter
skeleton for a requested element type, reusing the CLI template logic.

## Behaviour

- The skeleton shall include the type's required fields with placeholder values and the most
  common optional fields, so an LLM can fill it in and create a valid element in one step
  (pairing with `create_element`, REQ-TRS-MCP-005).
- An unknown element type shall return a structured error listing the recognised types.
