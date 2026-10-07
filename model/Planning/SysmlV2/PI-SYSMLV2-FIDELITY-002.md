---
type: PlanningItem
id: PI-SYSMLV2-FIDELITY-002
name: "Named control steps and dangling-succession handling"
status: done
itemType: task
parent: PI-SYSMLV2-FIDELITY-001
achieves: [REQ-TRS-SYSMLV2-060, REQ-TRS-SYSMLV2-061, REQ-TRS-SYSMLV2-062, REQ-TRS-SYSMLV2-063]
evidence:
  - path: "repo:crates/syscribe-model/tests/sysmlv2_fidelity.rs"
tags:
  - sysmlv2
---

`sysmlv2/ingest.rs` named-step recognition and `sysmlv2/export_behavior.rs`.
