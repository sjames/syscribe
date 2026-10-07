---
id: REQ-TRS-SYSMLV2-042
type: Requirement
name: MCP server shall expose a read-only export_sysml tool returning the SysML v2 text
status: verified
reqDomain: software
verificationMethod: test
---

The MCP server **shall** register a read-only `export_sysml {package?}` tool returning the same text as `export-sysml [<package>]` without writing to disk; an unknown package is a tool error.

**Source:** `REQ-TRS-SYSMLV2-042` (product model), `ADR-SYS-SYSMLV2-002`.
