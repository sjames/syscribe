---
type: PlanningItem
id: PI-HARA-230
name: "W800 skips QM hazardous events (GH #230)"
status: done
itemType: bug
achieves: [REQ-TRS-HARA-001]
evidence:
  - ref: TC-TRS-HARA-001
  - path: repo:crates/syscribe-model/src/validator.rs
tags:
  - safety
---

Requirement and test case written first; the test in `validator.rs` failed before the change.
