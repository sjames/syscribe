---
id: FTE-GTI-008
type: FaultTreeEvent
eventKind: basic
name: "Hardware-path voltage sensing reads nominal"
ref: Behavior::Parts::SensorPart
failureRate: 1.0e-6
rateSource: "illustrative proposal for architecture comparison (FT-GTI-001)"
rateStatus: unverified
injection: [{kind: corrupt, port: out, field: v, value: "1.0", part: sensorHw}]
---

Hardware-path voltage sensing reads nominal.
