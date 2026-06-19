---
type: Diagram
name: AvionicsBayIBD
diagramKind: IBD
svgMode: companion
pumlMode: companion
pumlFile: ./AvionicsBayIBD.puml
subject: UAV::Avionics::AvionicsBay
shapes:
  s-pdu:  {ref: "UAV::Power::PowerDistributionUnit",     kind: Part}
  s-fc:   {ref: "UAV::Avionics::FlightController",       kind: Part}
  s-prop: {ref: "UAV::Propulsion::PropulsionSystem",     kind: Part}
  s-imu:  {ref: "UAV::Avionics::IMU",                   kind: Part}
  s-gps:  {ref: "UAV::Avionics::GPSReceiver",           kind: Part}
  s-gcs:  {ref: "GroundStation::GroundControlStation",  kind: Part}
edges:
  e-pdu-fc:   {source: s-pdu,  target: s-fc,   kind: flow}
  e-pdu-imu:  {source: s-pdu,  target: s-imu,  kind: flow}
  e-pdu-gps:  {source: s-pdu,  target: s-gps,  kind: flow}
  e-fc-prop:  {source: s-fc,   target: s-prop, kind: flow}
  e-fc-gcs:   {source: s-fc,   target: s-gcs,  kind: flow}
---

<img src="AvionicsBayIBD.svg" alt="Avionics Bay Internal Block Diagram" width="100%">

![AvionicsBayIBD](./AvionicsBayIBD.svg)

Internal block diagram of the avionics bay, showing how power flows from the PowerDistributionUnit to all onboard subsystems and how the FlightController exchanges control and telemetry signals with external elements.

The PowerDistributionUnit fans power out to the FlightController, IMU, and GPSReceiver via their `powerIn` ports. The FlightController drives the PropulsionSystem via its `controlOut` port and streams telemetry to the GroundControlStation via `telemetryOut`.
