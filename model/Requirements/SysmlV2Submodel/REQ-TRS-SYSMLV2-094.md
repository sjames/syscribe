---
type: Requirement
id: REQ-TRS-SYSMLV2-094
name: "Snapshot and timeslice occurrences map to the native portion kind"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

`snapshot occurrence o : T;` and `timeslice occurrence o;` shall be ingested as an `Occurrence` (or `EventOccurrence`) with `isPortion: true` and `portionKind: snapshot`/`timeslice` (spec 3.2), and `export-sysml` shall write an occurrence usage with `isPortion: true` with that portion keyword (`timeslice` when `portionKind:` is absent), after `individual` when both apply.
