---
type: PlanningItem
id: PI-CLIFIX-231
name: "CLI output defects (GH #231)"
status: done
itemType: bug
achieves: [REQ-TRS-CLIFIX-001]
evidence:
  - ref: TC-TRS-CLIFIX-001
  - path: repo:crates/syscribe/tests/cli_output_fixes.rs
tags:
  - cli
---

Requirement and test first; fixes in `bcov.rs` and `query.rs`.
