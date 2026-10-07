---
type: Requirement
id: REQ-TRS-SYSMLV2-089
name: "A while loop keeps its until condition"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

`while c { ... } until d;` shall be ingested as a `LoopAction` with `loopKind: while`, `condition: c` and the additive `untilCondition: d` sub-field (spec 8.7.6), and `export-sysml` shall write such an entry back as that statement.
