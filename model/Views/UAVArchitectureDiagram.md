---
type: Diagram
name: UAVArchitectureDiagram
diagramKind: IBD
svgMode: companion
pumlMode: companion
pumlFile: ./UAVArchitectureDiagram.puml
subject: UAV::UAVSystem
shapes:
  s-bat:   {ref: "UAV::Power::BatteryPack",                  kind: Part}
  s-pdu:   {ref: "UAV::Power::PowerDistributionUnit",        kind: Part}
  s-avi:   {ref: "UAV::Avionics::AvionicsBay",               kind: Part}
  s-fc:    {ref: "UAV::Avionics::FlightController",          kind: Part}
  s-prop:  {ref: "UAV::Propulsion::PropulsionSystem",        kind: Part}
  s-gcs:   {ref: "GroundStation::GroundControlStation",      kind: Part}
edges:
  e-bat-pdu:   {source: s-bat,  target: s-pdu,  kind: flow}
  e-pdu-avi:   {source: s-pdu,  target: s-avi,  kind: flow}
  e-pdu-prop:  {source: s-pdu,  target: s-prop, kind: flow}
  e-fc-gcs:    {source: s-fc,   target: s-gcs,  kind: flow}
---

![UAVArchitectureDiagram](./UAVArchitectureDiagram.svg)

Power and data flow across the UAV's main subsystems. The battery pack supplies the power distribution unit, which fans out to the avionics bay and propulsion system. The avionics bay streams telemetry to the ground control station via the UAV's outbound telemetry port.

The two leaf requirements shown — endurance (REQ-UAV-ENDUR-001) and data link range (REQ-UAV-COMM-001) — are satisfied by BatteryPack and AvionicsBay respectively.
