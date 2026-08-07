---
type: PlanningItem
id: PI-HPLE-QUAL-001
name: "Qual test cases TC-TRS-HPLE-* wired into the qual harness"
status: todo
itemType: task
parent: PI-HPLE-001
tags:
  - variability
  - multi-repo
---

Formal `qual/` coverage for `REQ-TRS-HPLE-001..005`, following the exact convention established
for `TC-TRS-SYSMLV2-*`/`TC-TRS-PLANITEM-*`: `qual/Requirements/`, `qual/TestCases/`,
`qual/fixtures/`, `qual/tests/tc/`. Full unfiltered `qual/tests/run_qual.sh` must stay green.
