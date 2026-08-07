---
type: PlanningItem
id: PI-HPLE-EXAMPLE-001
name: "Build a comprehensive worked example (multi-tier consolidation)"
status: todo
itemType: task
parent: PI-HPLE-001
tags:
  - variability
  - multi-repo
---

A realistic multi-repo example (mirroring `examples/sysmlv2-submodel/` and
`examples/planning-item/`'s structure): at least 3 tiers, `subConfigurations:` at the top two,
`parameterBindings:` closing some parameters at the immediate consolidating tier and deliberately
deferring others further up (demonstrating transitive pass-through), and at least one
intentionally-`default:`-supplied parameter that never needs external injection at all.
