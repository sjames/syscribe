---
id: REQ-TRS-SYSMLV2-028
type: Requirement
name: "A SysMLv2 verification def/verification maps to the native VerificationCaseDef/VerificationCase schema — subject, actors, objectives, result"
status: verified
reqDomain: software
verificationMethod: test
---

A `verification def` shall be synthesized into a native `VerificationCaseDef` element carrying
`supertype:`/`subject:`/`actors:`/`objectives:`/`result:`/`isAbstract:`/`doc`. A named
`verification` usage shall be synthesized into a native `VerificationCase` element carrying the same
fields with `typedBy:` in place of `supertype:`.

**Source:** `REQ-TRS-SYSMLV2-028` (product model), `ADR-SYS-SYSMLV2-001` addendum.
