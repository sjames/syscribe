---
type: Requirement
id: REQ-TRS-MCP-017
name: "Elements are exposed as MCP resources with reference completion"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
---

The MCP server shall expose each model element as an MCP resource and shall provide argument
completion for element references, so clients can browse and @-mention elements natively.

## Resources and completion

- Each element shall be readable as a resource under `syscribe://element/<qname>`, resolved via
  an MCP resource template, returning the element's structured detail (equivalent to
  `get_element` with `detail: true`).
- To avoid dumping thousands of URIs, the server need not statically list every element; the
  resource template plus completion is sufficient.
- The server shall offer completion for element-reference inputs (qualified names and stable
  ids) so a client can suggest valid targets as the user types.
