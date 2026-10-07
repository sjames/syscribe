---
type: Requirement
id: REQ-TRS-SYSMLV2-083
name: "occurrence, event occurrence, individual definitions and occurrence usages are ingested and exported"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

`occurrence def`, `individual def`, an `occurrence` usage and an `event occurrence` usage (with or without `individual`) shall be ingested as native `OccurrenceDef`, `IndividualDef`, `Occurrence` and `EventOccurrence` elements with `supertype:`/`typedBy:`, `multiplicity:`, `isAbstract:`, `isIndividual:` and doc, and `export-sysml` shall write the four native types back as those statements. An occurrence usage with a portion kind (`snapshot`/`timeslice`) stays unmapped and counted in `W543`.
