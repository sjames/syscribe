---
type: PlanningItem
id: PI-HPLE-BINDGUARD-001
name: "Reject illegal/redundant cross-tier parameterBindings: entries"
status: todo
itemType: task
parent: PI-HPLE-001
achieves: [REQ-TRS-HPLE-003]
tags:
  - variability
  - multi-repo
  - validation
---

Extend the existing single-model `E203`/`E204` reasoning ("must not bind a parameter of an
unselected feature" / "must not bind a fixed parameter") across the `subConfigurations:` boundary,
plus rejecting a double-bind of a parameter already closed by a nearer tier.
