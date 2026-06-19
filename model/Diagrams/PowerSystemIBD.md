---
type: Diagram
name: PowerSystemIBD
diagramKind: IBD
svgMode: companion
svgFile: ./PowerSystemIBD.svg
pumlMode: companion
pumlFile: ./PowerSystemIBD.puml
subject: UAV::Power::PowerSystem
shapes:
  s-boundary:
    ref: UAV::Power::PowerSystem
    kind: boundary
  s-battery:
    ref: UAV::Power::PowerSystem::battery
    kind: block
    parent: s-boundary
  s-pdu:
    ref: UAV::Power::PowerSystem::pdu
    kind: block
    parent: s-boundary
  s-battery-pout:
    ref: UAV::Power::PowerSystem::battery::powerOut
    kind: port
    parent: s-battery
  s-pdu-pin:
    ref: UAV::Power::PowerSystem::pdu::powerIn
    kind: port
    parent: s-pdu
  s-pdu-pout:
    ref: UAV::Power::PowerSystem::pdu::powerOut
    kind: port
    parent: s-pdu
  s-main-pout:
    ref: UAV::Power::PowerSystem::mainPowerOut
    kind: port
    parent: s-boundary
edges:
  e-batt-pdu:
    ref: UAV::Power::PowerSystem
    source: s-battery-pout
    target: s-pdu-pin
    kind: flow
  e-pdu-bind:
    ref: UAV::Power::PowerSystem
    source: s-pdu-pout
    target: s-main-pout
    kind: binding
---

<img src="./PowerSystemIBD.svg" alt="Power System Internal Block Diagram" width="100%"/>

Internal Block Diagram for `PowerSystem`. Shows `battery : BatteryPack` supplying DC power to `pdu : PowerDistributionUnit` via a `PowerConnectionDef` flow, with the PDU's output port bound to the system-level `mainPowerOut` port presented at the boundary.
