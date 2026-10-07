---
id: REQ-TRS-SYSMLV2-040
type: Requirement
name: Tool shall comment out unsupported elements as skipped and report exported and skipped counts
status: verified
reqDomain: software
verificationMethod: test
---

An element of an unsupported type **shall** be replaced by `// skipped: <qname> (<type>)`, counted per type in the export report and trailing summary, without failing the export.

**Source:** `REQ-TRS-SYSMLV2-040` (product model), `ADR-SYS-SYSMLV2-002`.
