---
id: REQ-TRS-SYSMLV2-091
type: Requirement
name: "A succession's own type is ingested and exported"
status: verified
reqDomain: software
verificationMethod: test
---

A succession's own type (`succession s : T first a then b;`, `succession : T first a then b;`) shall be ingested as the additive `typedBy:` sub-field of the `successionConnections:` entry (spec 8.4.4), and `export-sysml` shall write it back, together with the entry's name and multiplicities, as that statement.

**Source:** `REQ-TRS-SYSMLV2-091` (product model), `ADR-SYS-SYSMLV2-001`.
