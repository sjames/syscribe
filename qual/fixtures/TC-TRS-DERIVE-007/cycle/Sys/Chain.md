---
type: PartDef
name: Chain
custom_fields:
  base: 4
derive:
  first: self.second + 1
  second: self.third * 2
  third: self.custom_fields.base
---

A valid (forward-referencing) chain — no E504; third = 4, second = 8, first = 9.
