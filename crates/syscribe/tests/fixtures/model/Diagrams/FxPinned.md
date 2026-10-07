---
type: Diagram
name: FxPinned
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
    kind: inheritance
layout:
  s-base:
    x: 40
    y: 40
    w: 160
    h: 50
  s-derived:
    x: 40
    y: 220
    w: 160
    h: 50
---

A fully pinned manifest diagram: every shape carries a `layout:` entry with a
measured size, so `export-html` draws it with the static SVG writer
(REQ-TRS-VIS-010, rule 1) and `diagram export --format svg` succeeds.
