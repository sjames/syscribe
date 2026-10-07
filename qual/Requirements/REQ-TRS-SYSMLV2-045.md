---
id: REQ-TRS-SYSMLV2-045
type: Requirement
name: Tool shall ingest a package-level SysMLv2 metadata def as a native MetadataDef
status: verified
reqDomain: software
verificationMethod: test
---

A `metadata def <Name> [:> <Super>] { ... }` member of a package shall become a `MetadataDef`
element (spec 8.15.1) carrying `supertype:` from the specialization, `isAbstract:` and the body's
`doc` text. A `metadata` *usage* has no sound native target and stays counted by `W543`; the
`@Syscribe*` annotations keep their existing dedicated lift.

**Source:** `REQ-TRS-SYSMLV2-045` (product model), `ADR-SYS-SYSMLV2-001`.
