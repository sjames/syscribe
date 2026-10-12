---
type: PartDef
name: Sys
features:
  - {name: src, typedBy: P::Src}
  - {name: mon, typedBy: P::Mon}
connections:
  - {from: src.out, to: mon.inp}
  - {from: mon.inp, to: src.out}
---
Sys part.
