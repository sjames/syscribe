---
type: PlanningItem
id: PI-MCP-MEM-002
name: "MCP memory, second round: one model in memory during a guarded write, and the live model's findings kept between writes"
status: done
itemType: feature
achieves: [REQ-TRS-MCP-MEM-000]
tags:
  - mcp
  - memory
evidence:
  - path: repo:crates/syscribe-model/src/mutate/guard.rs
  - path: repo:crates/syscribe/tests/mcp_memory.rs
  - path: repo:crates/syscribe-server/tests/release_model.rs
---

`compute_baseline` and `guarded_write_cached`: the baseline is kept in the MCP and web stores, a commit
stores the candidate's findings as the next one, and a model of 3,000 elements or more is dropped while
the candidate is built. Measured results are in `docs/design/mcp-memory.md`.
