---
id: REQ-TRS-SYSMLV2-092
type: Requirement
name: "Structural successions in part bodies are ingested and exported"
status: verified
reqDomain: software
verificationMethod: test
---

A `first a then b;` or `succession s : T [m] first [x] a then [y] b;` member of a `part def` or `part` usage body shall be ingested as a `successionConnections:` entry on the `PartDef`/`Part` (spec 8.4.4 permits the field on them), and `export-sysml` shall write such entries inside the part body when they read back identically, degrading otherwise to a comment counted by the export report.

**Source:** `REQ-TRS-SYSMLV2-092` (product model), `ADR-SYS-SYSMLV2-001`.
