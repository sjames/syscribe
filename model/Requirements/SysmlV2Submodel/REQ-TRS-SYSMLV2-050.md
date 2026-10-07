---
type: Requirement
id: REQ-TRS-SYSMLV2-050
name: "export-sysml emits an attribute unit as a SysML literal-with-unit instead of a comment"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2ExportADR
tags:
  - sysmlv2
  - export
---

An inline `features:` attribute entry with a numeric `value:` and a `unit:` shall export as
`attribute n : T = 5 [kg];` (SysML literal with unit). A `unit:` without a numeric value keeps the
`// unit: u` trailing comment, because SysML has no unit-only attribute form.
