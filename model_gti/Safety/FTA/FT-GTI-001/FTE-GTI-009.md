---
id: FTE-GTI-009
type: FaultTreeEvent
eventKind: basic
name: "Common cause: both voltage-sensing chains read nominal"
ref: Behavior::Parts::SensorPart
failureRate: 1.0e-8
rateSource: "illustrative proposal for architecture comparison (FT-GTI-001)"
rateStatus: unverified
injection: [{kind: corrupt, port: out, field: v, value: "1.0", part: sensorSw}, {kind: corrupt, port: out, field: v, value: "1.0", part: sensorHw}]
---

A fault that blinds both sensing chains together, for example a wiring error at the grid connection point or a shared reference. The two chains are independent (REQ-GTI-HW-008), so only a common cause defeats them both. Rate is an illustrative proposal, **unverified**. Both chains then read a fixed value, so the perturbation-response check (`REQ-GTI-SW-008`) sees no response to its injected current and reports the grid lost: the event is a cut set of order 2, together with the failure of the check (FTE-GTI-010). The executable campaign reports the single event as survived and the pair as defeated.
