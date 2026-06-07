---
type: PartDef
name: FlightController
supertype: Parts::Part
domain: software
asilLevel: C
satisfies:
  - REQ-UAV-FC-001
  - REQ-UAV-SAFE-001
  - REQ-UAV-COMM-001
implementedBy:
  - repo:firmware/flight_control/mod.rs
metadata:
  - type: Metadata::DALAnnotation
    level: 1
    rationale: "Flight controller failure is a catastrophic failure condition per DO-178C"
  - type: Metadata::ModelStatus
    status: approved
    reviewer: "Chief Systems Engineer"
features:
  - name: flightMode
    typedBy: Enumerations::FlightMode
    value: "FlightMode::hover"
    valueKind: initial
  - name: armStatus
    typedBy: Enumerations::ArmStatus
    value: "ArmStatus::disarmed"
    valueKind: initial
  - name: powerIn
    type: Port
    typedBy: Interfaces::PowerPortReceiverDef
    direction: in
  - name: controlOut
    type: Port
    typedBy: Interfaces::ControlPortDef
    direction: out
    ports:
      - name: throttle
        direction: out
      - name: attitude
        direction: out
      - name: yaw
        direction: out
  - name: telemetryOut
    type: Port
    typedBy: Interfaces::TelemetryPortDef
    direction: out
exhibitsStates:
  - Behavior::FlightStates
performs:
  - name: runMission
    typedBy: Behavior::MissionExecution
    multiplicity: "1"
rep: "FlightController : PartDef { ... }"
---

Embedded flight controller running the autopilot stack. Manages flight mode transitions, motor mixing, sensor fusion, and telemetry generation. DAL A per DO-178C.
