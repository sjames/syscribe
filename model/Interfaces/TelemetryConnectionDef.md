---
type: ConnectionDef
name: TelemetryConnectionDef
supertype: Connections::BinaryConnection
ends:
  - name: transmitter
    typedBy: Interfaces::TelemetryPortDef
  - name: receiver
    typedBy: Interfaces::TelemetryPortReceiverDef
---

A binary connection carrying telemetry packets from the UAV to the ground control station.
