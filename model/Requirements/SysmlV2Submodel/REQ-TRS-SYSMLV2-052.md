---
type: Requirement
id: REQ-TRS-SYSMLV2-052
name: "export-sysml emits constraint and calc parameters and expression bodies"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2ExportADR
tags:
  - sysmlv2
  - export
---

`ConstraintDef`/`Constraint` and `CalculationDef`/`Calculation` elements shall export their
`parameters:` entries (`in`/`out`/`inout x : T;`, `return r : T;` for `direction: return`) and
their `expression:`/`body:` text, one statement per line, inside the body, so a model ingested by
`REQ-TRS-SYSMLV2-033`/`-034` and exported again re-ingests with equal parameters and expression
text. Placeholder text such as `<conditional expression>` is never emitted as source; it becomes a
comment.
