---
type: Requirement
id: REQ-TRS-SYSMLV2-063
name: "Round-trip tests verify cross-entry consistency of exported behaviour bodies"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

The export parse-back tests shall check, beyond per-entry equality, that every emitted \`first … then …;\` names only steps that are exported in the same body, and that re-ingesting a whole exported body yields successions and control nodes equal to the exported subset, so entry-level verification cannot hide a dangling reference between entries.
