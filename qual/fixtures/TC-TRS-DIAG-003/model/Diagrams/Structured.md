---
type: Diagram
name: Structured
diagramKind: BDD
subject: Arch::Vehicle
shapes:
  s-vehicle: {ref: "Arch::Vehicle", kind: PartDef}
  s-engine:  {ref: "Arch::Engine", kind: PartDef}
edges:
  e-comp: {source: s-vehicle, target: s-engine, kind: composition}
layout:
  s-vehicle: {x: 20, y: 20, w: 160, h: 80}
  s-engine:  {x: 20, y: 160, w: 160, h: 80}
---

Structured SVG diagram: the server builds the SVG from the manifest and
`layout:`; the body carries no inline SVG.
