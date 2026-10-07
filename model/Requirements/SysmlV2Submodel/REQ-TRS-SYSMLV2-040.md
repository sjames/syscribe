---
type: Requirement
id: REQ-TRS-SYSMLV2-040
name: "The SysML v2 export comments out every element of an unsupported type with a skipped marker and reports exported and skipped counts per type"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2ExportADR
tags:
  - sysmlv2
  - export
  - loss-accounting
---

An element whose type is outside the supported set (for example `TestCase`, `ADR`, `PlanningItem`,
`FeatureDef`) shall not be rendered. In the position it would have occupied the export shall emit a
`// skipped: <qname> (<type>)` line comment, and the export report (returned by the library, appended as
a trailing `//` comment block of the text, and summarised on stderr by the CLI) shall count elements
exported and skipped per element type. The descendants of a skipped element are skipped with it. A
skipped element never fails the export.
