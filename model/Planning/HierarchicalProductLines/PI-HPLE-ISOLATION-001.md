---
type: PlanningItem
id: PI-HPLE-ISOLATION-001
name: "Confirm no mechanism lets a lower tier reference or become aware of a higher tier"
status: todo
itemType: task
parent: PI-HPLE-001
achieves: [REQ-TRS-HPLE-005]
tags:
  - variability
  - multi-repo
---

Architectural/structural rather than independently unit-testable in isolation, per
`REQ-TRS-HPLE-005`'s own scope — its observable consequence is that `PI-HPLE-SUBCONFIG-001`,
`PI-HPLE-PARAMBIND-001`, and `PI-HPLE-BINDGUARD-001`'s own tests never require, accept, or resolve
an upward-pointing reference from a descendant. Evidence here should point at a specific
regression test (in one of those three items' own test suites, or a dedicated one) that positively
confirms `bindTo:` cannot be repurposed to cross a `subConfigurations:` boundary, rather than
inventing new mechanism-building work of its own.
