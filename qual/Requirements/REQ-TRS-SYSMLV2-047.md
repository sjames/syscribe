---
id: REQ-TRS-SYSMLV2-047
type: Requirement
name: Tool shall lift doc on a SysMLv2 requirement def or requirement onto the element doc text
status: verified
reqDomain: software
verificationMethod: test
---

`doc /* ... */` blocks in the body of a `requirement def` or `requirement` usage shall become the
element's doc text, exactly like every other doc lift (`REQ-TRS-SYSMLV2-009`): trimmed, joined by a
blank line, the `@Syscribe*:` directive handling unchanged.

**Source:** `REQ-TRS-SYSMLV2-047` (product model), `ADR-SYS-SYSMLV2-001`.
