---
id: REQ-TRS-SYSMLV2-032
type: Requirement
name: MCP server shall expose a read-only sysml_submodels tool
status: verified
reqDomain: software
verificationMethod: test
---

The MCP server **shall** register a read-only `sysml_submodels` tool returning the same JSON as `sysml --json` without writing to disk.

**Source:** `REQ-TRS-SYSMLV2-032` (product model), `ADR-SYS-SYSMLV2-001` addendum.
