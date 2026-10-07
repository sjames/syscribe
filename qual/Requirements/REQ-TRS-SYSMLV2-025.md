---
id: REQ-TRS-SYSMLV2-025
type: Requirement
name: "A SysMLv2 enum def/enum maps to the native EnumerationDef/Enumeration schema — values, supertype, typedBy"
status: verified
reqDomain: software
verificationMethod: test
---

An `enum def` shall be synthesized into a native `EnumerationDef` element carrying `supertype:`/
`values:` (each entry `{name: ...}`). A named `enum` usage shall be synthesized into a native
`Enumeration` element carrying `typedBy:`/`doc`.

**Source:** `REQ-TRS-SYSMLV2-025` (product model), `ADR-SYS-SYSMLV2-001` addendum.
