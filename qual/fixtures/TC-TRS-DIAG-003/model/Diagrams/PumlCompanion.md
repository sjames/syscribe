---
type: Diagram
name: PumlCompanion
diagramKind: BDD
pumlMode: companion
pumlFile: ./PumlCompanion.puml
subject: Arch::Vehicle
shapes:
  s-vehicle: {ref: "Arch::Vehicle", kind: PartDef}
  s-engine:  {ref: "Arch::Engine", kind: PartDef, parent: s-vehicle}
edges:
  e-comp: {source: s-vehicle, target: s-engine, kind: composition}
---

PlantUML companion diagram: the manifest feeds the generated `.puml`; there is
no inline SVG by design.

![PumlCompanion](./PumlCompanion.svg)
