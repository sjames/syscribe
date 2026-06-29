---
type: Diagram
name: FxBlock
diagramKind: BDD
subject: Parts
shapes:
  s-base:
    ref: Parts::Base
    kind: PartDef
  s-derived:
    ref: Parts::Derived
    kind: PartDef
edges:
  e-spec:
    source: s-derived
    target: s-base
    kind: supertype
layout:
  s-base:
    x: 40
    y: 40
  s-derived:
    x: 40
    y: 220
---

Block diagram of the fixture parts (SVG-rendered).
