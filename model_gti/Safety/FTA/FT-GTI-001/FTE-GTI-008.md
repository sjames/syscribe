---
id: FTE-GTI-008
type: FaultTreeEvent
eventKind: basic
name: "Hardware-path voltage sensing reads nominal"
ref: Behavior::Parts::SensorPart
failureRate: 1.0e-6
injection: [{kind: corrupt, port: out, field: v, value: "1.0", part: sensorHw}]
---

Hardware-path voltage sensing reads nominal.
