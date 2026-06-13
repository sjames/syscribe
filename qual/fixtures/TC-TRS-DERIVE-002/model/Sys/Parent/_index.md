---
type: PartDef
name: Parent
derive:
  totalWcet: "sum(children.custom_fields.wcet)"
  childCount: "count(children)"
---

Parent element that aggregates child WCET values.
