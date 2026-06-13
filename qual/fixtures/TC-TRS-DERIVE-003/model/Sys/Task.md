---
type: PartDef
name: Task
custom_fields:
  budget: 100
  used: 60
derive:
  headroom: "self.custom_fields.budget - self.custom_fields.used"
  ratio: "self.custom_fields.used / (self.custom_fields.budget ?? 1)"
---

A task with budget tracking via derived fields.
