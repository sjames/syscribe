---
type: FlowDef
name: TelemetryFlowDef
supertype: Flows::FlowConnection
itemType: Items::TelemetryPacket
ends:
  - name: txPort
    typedBy: Interfaces::TelemetryPortDef
    direction: out
    isEnd: true
  - name: rxPort
    typedBy: Interfaces::TelemetryPortReceiverDef
    direction: in
    isEnd: true
---

A directed telemetry data flow from the UAV transmitter port to the ground station receiver port.
