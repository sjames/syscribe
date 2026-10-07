---
type: Requirement
id: REQ-TRS-SYSMLV2-059
name: "syscribe sysml and sysml_submodels count unresolved package-level satisfy statements and unresolved includes as unmapped"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - report
---

The per-file and per-submodel unmapped counts reported by `syscribe sysml`/`sysml_submodels` shall equal the counts ingestion raises in `W543`, including package-level `satisfy R by X;` statements whose subject does not resolve and includes that do not resolve (both previously visible only in the advisory, never in the report). The report shall obtain them from the ingestion pass itself rather than re-deriving them.
