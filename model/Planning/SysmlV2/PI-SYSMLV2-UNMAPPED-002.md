---
type: PlanningItem
id: PI-SYSMLV2-UNMAPPED-002
name: "Emit W543 per file with unmapped-construct counts, with tests and docs"
status: done
itemType: task
parent: PI-SYSMLV2-UNMAPPED-001
achieves: [REQ-TRS-SYSMLV2-030]
evidence:
  - path: "repo:crates/syscribe-model/tests/sysmlv2_unmapped.rs"
tags:
  - sysmlv2
  - validation
---

Count unmapped package-level constructs per file in `sysmlv2/ingest.rs`, push one `W543` finding
onto the submodel owner, register the code in the validator mapping, add the rows to
`prompts/spec/validation.md` and `docs/validation/rules.md`, document it in
`docs/model-guide/sysmlv2-submodel.md`, and cover it with `TC-TRS-SYSMLV2-030`.
