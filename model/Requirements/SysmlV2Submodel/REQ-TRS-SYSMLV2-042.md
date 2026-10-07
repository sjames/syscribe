---
type: Requirement
id: REQ-TRS-SYSMLV2-042
name: "A read-only MCP tool export_sysml returns the same SysML v2 text as syscribe export-sysml without writing to disk"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2ExportADR
tags:
  - sysmlv2
  - export
  - mcp
---

The MCP server shall register a read-only tool `export_sysml` (optional `package` argument,
`readOnlyHint` true) returning the SysML v2 text `syscribe -m <root> export-sysml [<package>]` prints,
computed from the in-memory model store. The tool shall never write to the model directory or any other
path; an unresolvable `package` is a tool error.
