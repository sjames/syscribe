---
type: Requirement
id: REQ-TRS-SYSMLV2-049
name: "export-sysml emits subsets and redefines on usages and round-trips multiplicity"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2ExportADR
tags:
  - sysmlv2
  - export
---

A usage with `subsets:` shall export `:> a, b` and one with `redefines:` `:>> a` after its typing
and multiplicity, so that re-ingesting the text yields the same `subsets:`, `redefines:` and
`multiplicity:` values. `multiplicity:` stays emitted as `[m]` and is covered by a parse-back test.
