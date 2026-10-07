---
type: PlanningItem
id: PI-SYSMLV2-INSPECT-003
name: "Register the read-only MCP sysml_submodels tool"
status: done
itemType: task
parent: PI-SYSMLV2-INSPECT-001
achieves: [REQ-TRS-SYSMLV2-032]
evidence:
  - path: "repo:crates/syscribe/tests/sysml_inspect.rs"
tags:
  - sysmlv2
  - mcp
---

`sysml_submodels` tool in `crates/syscribe/src/mcp/mod.rs`, returning `submodels_json` from the
store's elements; listed in the MCP tool docs.
