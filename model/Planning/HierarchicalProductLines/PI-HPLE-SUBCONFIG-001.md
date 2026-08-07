---
type: PlanningItem
id: PI-HPLE-SUBCONFIG-001
name: "subConfigurations: field, resolution, and peer-Configuration validity gate"
status: todo
itemType: task
parent: PI-HPLE-001
achieves: [REQ-TRS-HPLE-001]
tags:
  - variability
  - multi-repo
---

Add `subConfigurations:` to `Configuration`'s schema; resolve each entry to a real `Configuration`
(local or `repoImports:`-mounted); require the resolved `Configuration` to itself be internally
valid (SAT-clean) before it can be consolidated.
