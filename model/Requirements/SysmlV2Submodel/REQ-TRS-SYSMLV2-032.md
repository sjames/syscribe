---
type: Requirement
id: REQ-TRS-SYSMLV2-032
name: "A read-only MCP tool sysml_submodels returns the same SysMLv2 submodel inspection data as syscribe sysml --json"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-031]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - mcp
---

The MCP server shall register a read-only tool `sysml_submodels` (no arguments, `readOnlyHint`
true) that returns exactly the JSON document `syscribe -m <root> sysml --json` prints, computed
from the in-memory model store. The tool shall never write to the model directory.
