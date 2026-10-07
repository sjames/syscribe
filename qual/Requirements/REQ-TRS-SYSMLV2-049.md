---
id: REQ-TRS-SYSMLV2-049
type: Requirement
name: export-sysml shall emit subsets and redefines on usages and round-trip multiplicity
status: verified
reqDomain: software
verificationMethod: test
---

A usage with `subsets:` shall export `:> a, b` and one with `redefines:` `:>> a` after its typing
and multiplicity, so that re-ingesting the text yields the same `subsets:`, `redefines:` and
`multiplicity:` values. `multiplicity:` stays emitted as `[m]` and is covered by a parse-back test.

**Source:** `REQ-TRS-SYSMLV2-049` (product model), `ADR-SYS-SYSMLV2-002`.
