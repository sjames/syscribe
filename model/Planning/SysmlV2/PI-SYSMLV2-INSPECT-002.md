---
type: PlanningItem
id: PI-SYSMLV2-INSPECT-002
name: "Implement sysmlv2::report in syscribe-model and the syscribe sysml CLI command"
status: done
itemType: task
parent: PI-SYSMLV2-INSPECT-001
achieves: [REQ-TRS-SYSMLV2-031]
evidence:
  - path: "repo:crates/syscribe-model/tests/sysmlv2_report.rs"
  - path: "repo:crates/syscribe/tests/sysml_inspect.rs"
tags:
  - sysmlv2
  - cli
---

Reusable data gathering in `crates/syscribe-model/src/sysmlv2/report.rs`, text/JSON formatting in
`crates/syscribe/src/linktypes.rs` (`cmd_sysml`), the `sysml` dispatch in `main.rs`, the help topic
`prompts/help/sysml.md`, and docs in `docs/cli/index.md` and `docs/model-guide/sysmlv2-submodel.md`.
