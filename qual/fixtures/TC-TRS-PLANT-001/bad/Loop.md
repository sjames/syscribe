---
type: PlantDef
name: Loop
outputs:
  - {name: y}
  - {name: w}
equations:
  - "y := w"
  - "w := y"
---

An algebraic loop.
