---
type: PartDef
name: UAVSystem
supertype: Parts::Part
isAbstract: true
domain: system
satisfies:
  - REQ-UAV-MASS-001
features:
  - name: airframe
    typedBy: UAV::Airframe
    multiplicity: "1"
  - name: totalMassKg
    typedBy: ScalarValues::Real
    unit: SI::kg
    isDerived: true
  - name: telemetryOut
    type: Port
    typedBy: Interfaces::TelemetryPortDef
    direction: out
exhibitsStates:
  - Behavior::FlightStates
bindingConnections:
  - left: airframe.telemetryOut
    right: telemetryOut
---

Abstract top-level UAV system definition. Concrete system configurations specialize this definition. All safety requirements are scoped to this element.
