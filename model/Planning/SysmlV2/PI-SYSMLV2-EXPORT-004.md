---
type: PlanningItem
id: PI-SYSMLV2-EXPORT-004
name: "Register the read-only MCP export_sysml tool"
status: done
itemType: task
parent: PI-SYSMLV2-EXPORT-001
achieves: [REQ-TRS-SYSMLV2-042]
evidence:
  - path: "repo:crates/syscribe/tests/sysml_export.rs"
tags:
  - sysmlv2
  - export
---

`export_sysml` in `crates/syscribe/src/mcp/mod.rs`, listed in the MCP tool docs and the `mcp` help topic.
