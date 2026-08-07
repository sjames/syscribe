---
type: PlanningItem
id: PI-HPLE-001
name: "Implement hierarchical product-line composition (subConfigurations)"
status: in_progress
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
