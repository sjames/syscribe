---
type: Requirement
id: REQ-TRS-BUILD-003
name: "Configuration shall support an optional buildOverrides field"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BUILD-000]
breakdownAdr: Decisions::BuildExportADR
tags:
  - schema
  - build-integration
---

A `Configuration` element shall support an optional `buildOverrides:` field — a flat
key/value mapping of variable names to scalar values. Entries in `buildOverrides:` are
applied after all feature-derived and parameter-derived variables have been resolved,
overwriting any earlier value for the same variable name.

`buildOverrides:` is intended for variables that are configuration-specific rather than
feature-derived: version strings, product SKU identifiers, OEM branding constants, and
similar per-product values that have no corresponding `FeatureDef` parameter.

The field is optional; a `Configuration` without `buildOverrides:` remains fully valid.
