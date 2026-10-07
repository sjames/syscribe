---
type: PlanningItem
id: PI-SYSMLV2-CONSTRAINT-002
name: "Implement the constraint mapping with tests and docs"
status: done
itemType: task
parent: PI-SYSMLV2-CONSTRAINT-001
achieves: [REQ-TRS-SYSMLV2-033]
evidence:
  - path: "repo:crates/syscribe-model/tests/sysmlv2_constraints_calcs.rs"
tags:
  - sysmlv2
---

`ingest.rs` (`convert_constraint_def`/`convert_constraint_usage`, `constraint_body_fields`) in `crates/syscribe-model/src/sysmlv2/`, W543 accounting, tests in
`sysmlv2_constraints_calcs.rs`, and `docs/model-guide/sysmlv2-submodel.md` §22.
