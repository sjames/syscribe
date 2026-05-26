---
type: PartDef
name: GroundControlStation
supertype: Parts::Part
dependsOn:
  - Interfaces::TelemetryInterface
features:
  - name: console
    typedBy: GroundStation::OperatorConsole
    multiplicity: "1"
  - name: telemetryIn
    type: Port
    typedBy: Interfaces::TelemetryPortReceiverDef
    direction: in
connections:
  - typedBy: Interfaces::TelemetryConnectionDef
    ends:
      - end: transmitter
        binds: telemetryIn
      - end: receiver
        binds: console.telemetryIn
---

Ground control station receiving UAV telemetry and providing operator interface for mission command and monitoring.
