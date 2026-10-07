---
id: REQ-TRS-SYSMLV2-072
type: Requirement
name: "The export summary reports how many behavioural entries were not exported"
status: verified
reqDomain: software
verificationMethod: test
---

The `export-sysml` summary line (and the machine report) shall state the number of behaviour entries that degraded to comments, so a model owner can see the loss without reading the output. The count is zero for a model whose behaviour all reads back identically.

**Source:** `REQ-TRS-SYSMLV2-072` (product model), `ADR-SYS-SYSMLV2-002`.
