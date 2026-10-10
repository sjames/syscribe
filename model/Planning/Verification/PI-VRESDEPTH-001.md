---
type: PlanningItem
id: PI-VRESDEPTH-001
name: "verification-depth counts only non-failing, run tests (GH #257 b, c)"
status: done
itemType: bug
achieves: [REQ-TRS-VRES-002]
evidence:
  - ref: TC-TRS-VRES-002
  - path: repo:crates/syscribe/tests/verification_depth_results.rs
tags:
  - verification
---

Parts (b) and (c) of #257.
