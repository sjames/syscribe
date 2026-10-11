---
id: FTE-GTI-009
type: FaultTreeEvent
eventKind: basic
name: "Common cause: both voltage-sensing chains read nominal"
ref: Behavior::Parts::SensorPart
failureRate: 1.0e-8
injection: [{kind: corrupt, port: out, field: v, value: "1.0", part: sensorSw}, {kind: corrupt, port: out, field: v, value: "1.0", part: sensorHw}]
---

A fault that blinds both sensing chains together, for example a wiring error at the grid connection point or a shared reference. The two chains are independent (REQ-GTI-HW-008), so only a common cause defeats them both. Rate is an illustrative proposal, **unverified**. This event is a **single cut set**: the executable campaign reports it as defeated, which is the accepted-for-now residual risk of SG-GTI-001 (open decision, see the findings page).
