---
type: Diagram
name: Pinned
diagramKind: BDD
subject: UAV
shapes:
  s-fc:
    ref: UAV::Avionics::FlightController
    kind: PartDef
  s-safe:
    ref: Requirements::SafeLanding
    kind: Requirement
edges:
  e-sat:
    source: s-fc
    target: s-safe
    kind: satisfies
layout:
  s-fc:
    x: 40
    y: 220
    w: 180
    h: 60
  s-safe:
    x: 40
    y: 40
    w: 180
    h: 60
---
A fully pinned diagram (every shape has a `layout:` entry with a size), so
`diagram export --format svg` draws it and wraps each linked shape in a hyperlink.
