---
type: PortDef
name: TelemetryPortDef
supertype: Ports::Port
features:
  - name: packet
    typedBy: Items::TelemetryPacket
    direction: out
    multiplicity: "0..*"
operations:
  - name: requestLatest
    doc: "Synchronously return the most recent telemetry packet."
    isQuery: true
    isAsync: false
    parameters: []
    returnType: Items::TelemetryPacket
  - name: subscribe
    doc: "Register for periodic telemetry push at the given interval."
    isAsync: true
    parameters:
      - name: intervalMs
        typedBy: ScalarValues::Integer
        direction: in
---

Port definition for outgoing telemetry data. Components emitting telemetry expose this port.
