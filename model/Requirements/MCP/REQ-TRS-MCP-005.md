---
type: Requirement
id: REQ-TRS-MCP-005
name: "create_element tool creates a new element with guards"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - write
---

The MCP server shall expose a `create_element` write tool that creates a new model element
`.md` file from a requested qualified name, element type, frontmatter fields, and optional
documentation body, subject to write guards.

## Guards

- **No overwrite.** `create_element` shall refuse to create an element whose qualified name
  already exists; it shall never silently overwrite an existing file.
- **Identity rules.** The qualified name shall be checked against the basic-name grammar, and
  for id-identified element types the stable `id` shall be auto-allocated using the same
  next-id logic as the CLI so ids cannot collide.
- **Guarded commit.** The tool shall honour the common write-guard behaviour of
  REQ-TRS-MCP-008 (`dry_run` default, validation delta, new-error commit gate, store rebuild).
