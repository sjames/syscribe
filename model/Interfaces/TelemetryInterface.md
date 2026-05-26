---
type: InterfaceDef
name: TelemetryInterface
supertype: Connections::Interface
ends:
  - name: transmitter
    typedBy: Interfaces::TelemetryPortDef
    multiplicity: "1"
  - name: receiver
    typedBy: Interfaces::TelemetryPortReceiverDef
    multiplicity: "1"
---

Interface for unidirectional telemetry transmission from the UAV to the ground station.
