---
type: Diagram
name: UAVSystemBDD
diagramKind: BDD
svgMode: companion
svgFile: ./UAVSystemBDD.svg
subject: UAV::UAVSystem
shapes:
  s-uavsystem: {ref: "UAV::UAVSystem", kind: PartDef}
  s-airframe: {ref: "UAV::Airframe", kind: PartDef}
  s-propulsion: {ref: "UAV::Propulsion::PropulsionSystem", kind: PartDef}
  s-avionics: {ref: "UAV::Avionics::AvionicsBay", kind: PartDef}
  s-power: {ref: "UAV::Power::PowerSystem", kind: PartDef}
  s-payload: {ref: "UAV::Payload::PayloadBay", kind: PartDef}
  s-gcs: {ref: "GroundStation::GroundControlStation", kind: PartDef}
edges:
  e-airframe: {ref: "UAV::UAVSystem::airframe", source: s-uavsystem, target: s-airframe, kind: composition}
  e-propulsion: {ref: "UAV::UAVSystem", source: s-uavsystem, target: s-propulsion, kind: composition}
  e-avionics: {ref: "UAV::UAVSystem", source: s-uavsystem, target: s-avionics, kind: composition}
  e-power: {ref: "UAV::UAVSystem", source: s-uavsystem, target: s-power, kind: composition}
  e-payload: {ref: "UAV::UAVSystem", source: s-uavsystem, target: s-payload, kind: composition}
---

Block Definition Diagram showing the top-level structural decomposition of the UAVSystem into its constituent subsystem definitions.

<img src="./UAVSystemBDD.svg" alt="UAV System Block Definition Diagram" width="100%"/>
