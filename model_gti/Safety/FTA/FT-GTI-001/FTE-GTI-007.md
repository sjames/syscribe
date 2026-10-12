---
id: FTE-GTI-007
type: FaultTreeEvent
eventKind: basic
name: "Software-path voltage sensing reads nominal"
ref: Behavior::Parts::SensorPart
failureRate: 1.0e-6
rateSource: "illustrative proposal for architecture comparison (FT-GTI-001)"
rateStatus: unverified
injection: [{kind: corrupt, port: out, field: v, value: "1.0", part: sensorSw}]
---

Software-path voltage sensing reads nominal.
