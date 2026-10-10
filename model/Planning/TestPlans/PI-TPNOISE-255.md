---
type: PlanningItem
id: PI-TPNOISE-255
name: "TestPlan warning noise: W616 by member overlap, W615 per plan (GH #255, #260)"
status: done
itemType: bug
achieves: [REQ-TRS-TPNOISE-001, REQ-TRS-TPNOISE-002]
evidence:
  - ref: TC-TRS-TPNOISE-001
  - path: repo:crates/syscribe-model/tests/testplan_warning_noise.rs
tags:
  - testplan
---

Requirements and test first. `W616` now compares effective member sets, `W615` is one finding per plan.
