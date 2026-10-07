---
type: Requirement
id: REQ-TRS-SYSMLV2-072
name: "The export summary reports how many behavioural entries were not exported"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

The `export-sysml` summary line (and the machine report) shall state the number of behaviour entries that degraded to comments, so a model owner can see the loss without reading the output. The count is zero for a model whose behaviour all reads back identically.
