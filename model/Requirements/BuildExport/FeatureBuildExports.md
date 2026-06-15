---
type: Requirement
id: REQ-TRS-BUILD-001
name: "FeatureDef shall support an optional buildExports field"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BUILD-000]
breakdownAdr: Decisions::BuildExportADR
tags:
  - schema
  - build-integration
---

A `FeatureDef` element shall support an optional `buildExports:` field. When present,
it shall be a list of build variable declarations. Each entry shall declare:

- `var:` *(required)* — the variable name to emit in generated output.
- `whenSelected:` *(optional, default `1`)* — the value assigned to the variable when
  the feature is selected in the active configuration.
- `whenDeselected:` *(optional)* — the value assigned when the feature is not selected.
  When absent and the feature is deselected, the variable shall be **omitted** from the
  output entirely.

The `buildExports:` field is optional at both the element level and the entry level;
a `FeatureDef` without `buildExports:` remains fully valid.
