---
id: REQ-TRS-SYSMLV2-050
type: Requirement
name: export-sysml shall emit an attribute unit as a SysML literal-with-unit instead of a comment
status: verified
reqDomain: software
verificationMethod: test
---

An inline `features:` attribute entry with a numeric `value:` and a `unit:` shall export as
`attribute n : T = 5 [kg];` (SysML literal with unit). A `unit:` without a numeric value keeps the
`// unit: u` trailing comment, because SysML has no unit-only attribute form.

**Source:** `REQ-TRS-SYSMLV2-050` (product model), `ADR-SYS-SYSMLV2-002`.
