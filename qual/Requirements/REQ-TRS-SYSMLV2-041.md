---
id: REQ-TRS-SYSMLV2-041
type: Requirement
name: Tool shall produce deterministic SysML v2 output that re-parses through ingestion with kinds and qnames intact
status: verified
reqDomain: software
verificationMethod: test
---

Exporting twice **shall** give identical text, and re-importing it as a `sysmlSubmodel` **shall** succeed with every supported element returning with the same qualified name and kind (native Requirement returns as RequirementDef), for `examples/sysmlv2-submodel/` and a native-only fixture.

**Source:** `REQ-TRS-SYSMLV2-041` (product model), `ADR-SYS-SYSMLV2-002`.
