---
type: Diagram
name: SysBDD
diagramKind: BDD
subject: Sys
shapes:
  s-engine: Sys::Engine
  s-motor: {ref: Sys::Motor, kind: PartDef}
edges:
  e-inh: {source: s-motor, target: s-engine, kind: inheritance}
---

BDD fixture using the string shorthand and the map form.
