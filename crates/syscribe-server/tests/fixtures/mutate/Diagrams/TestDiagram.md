---
type: Diagram
name: TestDiagram
diagramKind: bdd
shapes:
  s-widget:
    ref: Basics::Widget
    kind: PartDef
layout:
  s-widget:
    x: 10
    y: 10
edges: {}
---

A minimal diagram fixture used by the mutate-endpoint integration tests to
verify shape/layout/edge sync (REQ-TRS-DE-003).
