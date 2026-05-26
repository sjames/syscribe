---
type: PartDef
name: Airframe
supertype: Parts::Part
features:
  - name: propulsion
    typedBy: UAV::Propulsion::PropulsionSystem
    multiplicity: "1"
  - name: avionics
    typedBy: UAV::Avionics::AvionicsBay
    multiplicity: "1"
  - name: power
    typedBy: UAV::Power::PowerSystem
    multiplicity: "1"
  - name: payload
    typedBy: UAV::Payload::PayloadBay
    multiplicity: "0..1"
  - name: telemetryOut
    type: Port
    typedBy: Interfaces::TelemetryPortDef
    direction: out
  - name: massKg
    typedBy: ScalarValues::Real
    unit: SI::kg
    isDerived: true
connections:
  - typedBy: Interfaces::PowerConnectionDef
    ends:
      - end: source
        binds: power.mainPowerOut
      - end: receiver
        binds: propulsion.powerIn
  - typedBy: Interfaces::PowerConnectionDef
    ends:
      - end: source
        binds: power.mainPowerOut
      - end: receiver
        binds: avionics.powerIn
  - typedBy: Interfaces::PowerConnectionDef
    ends:
      - end: source
        binds: power.mainPowerOut
      - end: receiver
        binds: payload.powerIn
flowConnections:
  - name: telemetryLink
    typedBy: Flows::TelemetryFlowDef
    kind: streaming
    from: avionics.telemetryOut
    to: telemetryOut
bindingConnections:
  - left: avionics.telemetryOut
    right: telemetryOut
---

Top-level airframe composing propulsion, avionics, power, and payload subsystems. All power distribution and telemetry routing is wired at this level.
