---
id: REQ-TRS-SYSMLV2-083
type: Requirement
name: "occurrence, event occurrence, individual definitions and occurrence usages are ingested and exported"
status: verified
reqDomain: software
verificationMethod: test
---

`occurrence def`, `individual def`, an `occurrence` usage and an `event occurrence` usage (with or without `individual`) shall be ingested as native `OccurrenceDef`, `IndividualDef`, `Occurrence` and `EventOccurrence` elements with `supertype:`/`typedBy:`, `multiplicity:`, `isAbstract:`, `isIndividual:` and doc, and `export-sysml` shall write the four native types back as those statements. An occurrence usage with a portion kind (`snapshot`/`timeslice`) stays unmapped and counted in `W543`.

**Source:** `REQ-TRS-SYSMLV2-083` (product model), `ADR-SYS-SYSMLV2-001`.
