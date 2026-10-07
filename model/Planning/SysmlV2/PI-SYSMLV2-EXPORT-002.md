---
type: PlanningItem
id: PI-SYSMLV2-EXPORT-002
name: "Implement sysmlv2::export in syscribe-model"
status: done
itemType: task
parent: PI-SYSMLV2-EXPORT-001
achieves: [REQ-TRS-SYSMLV2-038, REQ-TRS-SYSMLV2-039, REQ-TRS-SYSMLV2-040, REQ-TRS-SYSMLV2-041]
evidence:
  - path: "repo:crates/syscribe-model/tests/sysmlv2_export.rs"
tags:
  - sysmlv2
  - export
---

The writer in `crates/syscribe-model/src/sysmlv2/export.rs`: qname tree to nested packages, kind mapping, identifier quoting, skip accounting, determinism, with parse-back tests.
