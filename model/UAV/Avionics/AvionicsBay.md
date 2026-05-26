---
type: PartDef
name: AvionicsBay
supertype: Parts::Part
features:
  - name: flightController
    typedBy: UAV::Avionics::FlightController
    multiplicity: "1"
  - name: gpsReceiver
    typedBy: UAV::Avionics::GPSReceiver
    multiplicity: "1"
  - name: imu
    typedBy: UAV::Avionics::IMU
    multiplicity: "1"
  - name: powerIn
    type: Port
    typedBy: Interfaces::PowerPortReceiverDef
    direction: in
  - name: telemetryOut
    type: Port
    typedBy: Interfaces::TelemetryPortDef
    direction: out
connections:
  - from: flightController.controlOut
    to: gpsReceiver.powerIn
  - typedBy: Interfaces::PowerConnectionDef
    ends:
      - end: source
        binds: powerIn
      - end: receiver
        binds: flightController.powerIn
  - typedBy: Interfaces::PowerConnectionDef
    ends:
      - end: source
        binds: powerIn
      - end: receiver
        binds: gpsReceiver.powerIn
  - typedBy: Interfaces::PowerConnectionDef
    ends:
      - end: source
        binds: powerIn
      - end: receiver
        binds: imu.powerIn
bindingConnections:
  - left: flightController.telemetryOut
    right: telemetryOut
---

Avionics bay housing the flight controller, GPS receiver, and IMU. Distributes avionics power and routes telemetry to the airframe telemetry port.
