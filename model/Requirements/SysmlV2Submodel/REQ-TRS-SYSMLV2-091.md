---
type: Requirement
id: REQ-TRS-SYSMLV2-091
name: "A succession's own type is ingested and exported"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

A succession's own type (`succession s : T first a then b;`, `succession : T first a then b;`) shall be ingested as the additive `typedBy:` sub-field of the `successionConnections:` entry (spec 8.4.4), and `export-sysml` shall write it back, together with the entry's name and multiplicities, as that statement.
