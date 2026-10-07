---
type: Requirement
id: REQ-TRS-SYSMLV2-092
name: "Structural successions in part bodies are ingested and exported"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

A `first a then b;` or `succession s : T [m] first [x] a then [y] b;` member of a `part def` or `part` usage body shall be ingested as a `successionConnections:` entry on the `PartDef`/`Part` (spec 8.4.4 permits the field on them), and `export-sysml` shall write such entries inside the part body when they read back identically, degrading otherwise to a comment counted by the export report.
