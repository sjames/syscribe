---
id: FTE-GTI-001
type: FaultTreeEvent
eventKind: basic
name: "Gate drive stuck on"
ref: Behavior::Parts::PowerStagePart
failureRate: 2.0e-7
injection: [{kind: stuck, variable: pwm, value: "1"}]
---

Gate drive stuck on.
