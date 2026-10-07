---
type: Requirement
id: REQ-TRS-SYSMLV2-084
name: "A named dependency is ingested and exported as a Dependency element"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

`dependency <name> from a, b to c;` shall be ingested as a native `Dependency` element with `clients:` and `suppliers:` (resolved from the scope of the declaring package), and `export-sysml` shall write a native `Dependency` as that statement. An anonymous `dependency from a to b;` has no identity to synthesize an element against and stays unmapped and counted in `W543`.
