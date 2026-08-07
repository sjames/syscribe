---
type: PlanningItem
id: PI-HPLE-PARAMBIND-001
name: "parameterBindings: reaches transitively through a consolidated subtree via ordinary qname resolution"
status: todo
itemType: task
parent: PI-HPLE-001
achieves: [REQ-TRS-HPLE-002]
tags:
  - variability
  - multi-repo
---

Extend `parameterBindings:` resolution so a dotted key can target a parameter reachable through
`subConfigurations:` at any depth, using the parameter's ordinary already-mounted qname — no new
addressing syntax.
