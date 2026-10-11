---
id: FTE-GTI-007
type: FaultTreeEvent
eventKind: basic
name: "Software-path voltage sensing reads nominal"
ref: Behavior::Parts::SensorPart
failureRate: 1.0e-6
injection: [{kind: corrupt, port: out, field: v, value: "1.0", part: sensorSw}]
---

Software-path voltage sensing reads nominal.
