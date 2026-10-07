---
id: REQ-TRS-SYSMLV2-048
type: Requirement
name: Tool shall ingest multiplicity, subsets and redefines on SysMLv2 part, attribute, port and item usages
status: verified
reqDomain: software
verificationMethod: test
---

A `part`, `attribute`, `port` or `item` usage that declares a multiplicity (`[2]`, `[0..*]`, `[*]`),
`:>`/`subsets` or `:>>`/`redefines` shall carry them in the native `multiplicity:` (normalized text:
`2`, `0..*`, `*`), `subsets:` (list) and `redefines:` fields. Targets go through the existing
`E112`/`E113` structural-reference checks and the scoped resolver, like `typedBy:`.

**Source:** `REQ-TRS-SYSMLV2-048` (product model), `ADR-SYS-SYSMLV2-001`.
