---
id: REQ-TRS-SYSMLV2-055
type: Requirement
name: "An attribute usage with a literal value maps the value, and a literal-with-unit its unit, onto the Attribute element"
status: verified
reqDomain: software
verificationMethod: test
---

`attribute a : T = 12.5 [kg];` shall synthesize an `Attribute` carrying `value: 12.5` and `unit: kg`; a boolean, integer, real or string literal value without a unit carries `value:` only (string literals as their unquoted text). Non-literal value expressions are not mapped. `unit:` is a recognised element-level frontmatter field (the spec already defines it on inline `features:` entries), so it never raises `W047`.

**Source:** `REQ-TRS-SYSMLV2-055` (product model), `ADR-SYS-SYSMLV2-001`.
