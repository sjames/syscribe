---
id: REQ-TRS-SYSMLV2-026
type: Requirement
name: "A SysMLv2 case def/case maps to the native CaseDef/Case schema — subject, actors, objectives, result"
status: verified
reqDomain: software
verificationMethod: test
---

A `case def` shall be synthesized into a native `CaseDef` element carrying `supertype:`/`subject:`/
`actors:`/`objectives:`/`result:`/`isAbstract:`/`doc`. A named `case` usage shall be synthesized into
a native `Case` element carrying the same fields with `typedBy:` in place of `supertype:`.

**Source:** `REQ-TRS-SYSMLV2-026` (product model), `ADR-SYS-SYSMLV2-001` addendum.
