---
type: PlanningItem
id: PI-HPLE-001
name: "Implement hierarchical product-line composition (subConfigurations)"
status: done
itemType: feature
achieves: [REQ-TRS-HPLE-000]
tags:
  - variability
  - multi-repo
---

Top-level planning item for `ADR-SYS-HPLE-001`/`REQ-TRS-HPLE-000..005`: a `Configuration` can be
consolidated from already-configured lower-tier product-line models. Broken down into one child per
requirement, plus a comprehensive worked example and qual test coverage, mirroring the task
granularity used for the SysMLv2 submodel and native `PlanningItem` features themselves.

This is also the first real (non-`examples/`) use of `PlanningItem` in this repo's own model —
tracking its own sibling feature's implementation.

All seven children are `done`: `PI-HPLE-SUBCONFIG-001` (`subConfigurations:` field, resolution,
peer-validity gate — `E516`–`E518`), `PI-HPLE-PARAMBIND-001` (transitive `parameterBindings:`
resolution — `REQ-TRS-HPLE-002`), `PI-HPLE-BINDGUARD-001` (cross-tier binding legality — `E519`,
`E523`), `PI-HPLE-OPENPARAM-001` (opt-in open-parameter completeness — `W513`),
`PI-HPLE-ISOLATION-001` (lower-tier isolation, confirmed architecturally plus a dedicated
regression test), `PI-HPLE-EXAMPLE-001` (the 3-tier `examples/hple-multitier/` worked example), and
`PI-HPLE-QUAL-001` (`TC-TRS-HPLE-001..005`, full unfiltered `qual/tests/run_qual.sh` green). See
each child's own evidence for the detailed trail.
