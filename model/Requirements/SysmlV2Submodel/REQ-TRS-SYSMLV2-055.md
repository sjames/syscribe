---
type: Requirement
id: REQ-TRS-SYSMLV2-055
name: "An attribute usage with a literal value maps the value, and a literal-with-unit its unit, onto the Attribute element"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - ingest
---

`attribute a : T = 12.5 [kg];` shall synthesize an `Attribute` carrying `value: 12.5` and `unit: kg`; a boolean, integer, real or string literal value without a unit carries `value:` only (string literals as their unquoted text). Non-literal value expressions are not mapped. `unit:` is a recognised element-level frontmatter field (the spec already defines it on inline `features:` entries), so it never raises `W047`.

## Rationale

The ingested attribute previously kept only its type, so a numeric/unit-bearing attribute lost its data on the way in and `export-sysml` could not emit it. Mapping only literals keeps the `value:` field's meaning unambiguous (a literal, never an opaque expression).
