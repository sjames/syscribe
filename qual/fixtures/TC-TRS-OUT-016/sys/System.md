---
type: PartDef
name: System
features:
  - {name: a, typedBy: PartA}
  - {name: b, typedBy: PartB}
  - {name: c, typedBy: PartC}
connections:
  - {typedBy: IfaceAB, from: a.out, to: b.in}
  - {typedBy: IfaceBC, from: b.out, to: c.in}
---
A composite with three subparts wired A→B→C.
