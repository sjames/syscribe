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
    kind: inheritance
---

Block diagram of the fixture parts: a manifest with no `layout:`, so it is
not pinned — `export-html` and `--format svg` lay it out with the embedded
ELK (REQ-TRS-VIS-016).
