---
type: Diagram
name: PropulsionSystemBDD
diagramKind: BDD
svgMode: companion
svgFile: ./PropulsionSystemBDD.svg
pumlMode: companion
pumlFile: ./PropulsionSystemBDD.puml
subject: UAV::Propulsion
shapes:
  s-propulsion:
    ref: UAV::Propulsion::PropulsionSystem
    kind: PartDef
  s-motor:
    ref: UAV::Propulsion::Motor
    kind: PartDef
  s-rotor:
    ref: UAV::Propulsion::RotorAssembly
    kind: PartDef
edges:
  e-motor-comp:
    ref: UAV::Propulsion::RotorAssembly::motor
    source: s-rotor
    target: s-motor
    kind: composition
---

<img src="./PropulsionSystemBDD.svg" alt="Propulsion System Block Definition Diagram" width="100%"/>

Block Definition Diagram for the Propulsion subsystem. Shows the three classifier definitions — `PropulsionSystem` (abstract), `Motor`, and `RotorAssembly` — with their key features and the composition relationship expressing that each `RotorAssembly` owns exactly one `Motor`.

Concrete propulsion configurations (`QuadRotorConfig`, `HexRotorConfig`) are `Part` usages typed by `RotorAssembly` and are shown in the IBD, not here.
