---
type: PlanningItem
id: PI-SYSMLV2-GAPS-003
name: "Export subsets/redefines, unit, satisfy/verify statements and constraint/calc bodies"
status: done
itemType: task
parent: PI-SYSMLV2-GAPS-001
achieves: [REQ-TRS-SYSMLV2-049, REQ-TRS-SYSMLV2-050, REQ-TRS-SYSMLV2-051, REQ-TRS-SYSMLV2-052]
evidence:
  - path: "repo:crates/syscribe-model/tests/sysmlv2_gaps.rs"
tags:
  - sysmlv2
---

`crates/syscribe-model/src/sysmlv2/export.rs`, parse-back round-trip tests and docs.
