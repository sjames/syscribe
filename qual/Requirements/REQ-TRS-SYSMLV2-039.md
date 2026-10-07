---
id: REQ-TRS-SYSMLV2-039
type: Requirement
name: Tool shall emit valid SysML v2 identifiers, quoting non-basic names and reserved words
status: verified
reqDomain: software
verificationMethod: test
---

Every emitted name and reference segment **shall** be a bare basic name, or single-quoted when it is not a basic name or is a reserved word, escaping `\\` and `'`; stable ids of exported elements render as qualified names.

**Source:** `REQ-TRS-SYSMLV2-039` (product model), `ADR-SYS-SYSMLV2-002`.
