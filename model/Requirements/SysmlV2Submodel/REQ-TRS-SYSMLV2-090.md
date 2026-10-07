---
type: Requirement
id: REQ-TRS-SYSMLV2-090
name: "Control node parameter bodies are ingested and exported"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

The `in`/`out`/`inout` parameter declarations in the body of a `fork`, `join`, `decide` or `merge` node — in the standalone form and in `then fork f { ... }` — shall be ingested as the additive `parameters:` list of the node's `controlNodes:` entry (`{name, direction, typedBy?}`, spec 8.7.4), and `export-sysml` shall write them back as the node's body.
