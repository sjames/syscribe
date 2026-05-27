---
type: Diagram
name: AvionicsBayIBD
diagramKind: ibd
svgMode: companion
expose:
  - UAV::Power::PowerDistributionUnit
  - UAV::Avionics::FlightController
  - UAV::Propulsion::PropulsionSystem
  - UAV::Avionics::IMU
  - UAV::Avionics::GPSReceiver
  - GroundStation::GroundControlStation
edges:
  - from: {qname: "UAV::Power::PowerDistributionUnit", port: powerOut}
    to:   {qname: "UAV::Avionics::FlightController",   port: powerIn}
    kind: flow
  - from: {qname: "UAV::Power::PowerDistributionUnit", port: powerOut}
    to:   {qname: "UAV::Avionics::IMU",                port: powerIn}
    kind: flow
  - from: {qname: "UAV::Power::PowerDistributionUnit", port: powerOut}
    to:   {qname: "UAV::Avionics::GPSReceiver",        port: powerIn}
    kind: flow
  - from: {qname: "UAV::Avionics::FlightController",   port: controlOut}
    to:   {qname: "UAV::Propulsion::PropulsionSystem",  port: powerIn}
    kind: flow
  - from: {qname: "UAV::Avionics::FlightController",   port: telemetryOut}
    to:   {qname: "GroundStation::GroundControlStation", port: telemetryIn}
    kind: flow
---

<img src="AvionicsBayIBD.svg" alt="Avionics Bay Internal Block Diagram" width="100%">

Internal block diagram of the avionics bay, showing how power flows from the PowerDistributionUnit to all onboard subsystems and how the FlightController exchanges control and telemetry signals with external elements.

The PowerDistributionUnit fans power out to the FlightController, IMU, and GPSReceiver via their `powerIn` ports. The FlightController drives the PropulsionSystem via its `controlOut` port and streams telemetry to the GroundControlStation via `telemetryOut`.
