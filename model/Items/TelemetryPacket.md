---
type: ItemDef
name: TelemetryPacket
supertype: Items::Item
features:
  - name: sequenceNumber
    typedBy: ScalarValues::Integer
  - name: timestampMs
    typedBy: ScalarValues::Integer
  - name: payloadSize
    typedBy: ScalarValues::Integer
---

A framed telemetry packet transmitted from the UAV to the ground control station over the RF link.
