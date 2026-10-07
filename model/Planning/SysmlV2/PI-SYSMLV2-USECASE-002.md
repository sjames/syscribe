---
type: PlanningItem
id: PI-SYSMLV2-USECASE-002
name: "Implement the usecase mapping with tests and docs"
status: done
itemType: task
parent: PI-SYSMLV2-USECASE-001
achieves: [REQ-TRS-SYSMLV2-035]
evidence:
  - path: "repo:crates/syscribe-model/tests/sysmlv2_constraints_calcs.rs"
tags:
  - sysmlv2
---

`ingest.rs` (`convert_use_case_def`/`convert_use_case_usage`, reusing `case_body_fields`) in `crates/syscribe-model/src/sysmlv2/`, W543 accounting, tests in
`sysmlv2_constraints_calcs.rs`, and `docs/model-guide/sysmlv2-submodel.md` §22.
