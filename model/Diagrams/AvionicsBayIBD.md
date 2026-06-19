---
type: Diagram
name: AvionicsBayIBD
diagramKind: IBD
svgMode: companion
svgFile: ./AvionicsBayIBD.svg
pumlMode: companion
pumlFile: ./AvionicsBayIBD.puml
subject: UAV::Avionics::AvionicsBay
shapes:
  s-boundary: {ref: "UAV::Avionics::AvionicsBay", kind: boundary}
  s-fc: {ref: "UAV::Avionics::FlightController", kind: Part, parent: s-boundary}
  s-imu: {ref: "UAV::Avionics::IMU", kind: Part, parent: s-boundary}
  s-gps: {ref: "UAV::Avionics::GPSReceiver", kind: Part, parent: s-boundary}
  s-fc-power: {ref: "UAV::Avionics::FlightController::powerIn", kind: Port}
  s-fc-ctrl: {ref: "UAV::Avionics::FlightController::controlOut", kind: Port}
  s-fc-telem: {ref: "UAV::Avionics::FlightController::telemetryOut", kind: Port}
edges:
  e-power: {ref: "UAV::Avionics::AvionicsBay", source: s-fc-power, target: s-imu, kind: flowConnection}
  e-ctrl: {ref: "UAV::Avionics::AvionicsBay", source: s-fc-ctrl, target: s-gps, kind: flowConnection}
---

![AvionicsBayIBD](./AvionicsBayIBD.svg)

Internal Block Diagram showing the internal structure of the AvionicsBay, including part usages for FlightController, IMU, and GPSReceiver, with their interconnecting ports and flow connections.
