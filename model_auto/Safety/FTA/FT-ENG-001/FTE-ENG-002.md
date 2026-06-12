---
type: FaultTreeEvent
id: FTE-ENG-002
name: Throttle position sensor dual-track failure
eventKind: basic
ref: System::Sensors::ThrottlePositionSensor
failureRate: 5.0e-7
probability: 5.0e-4
---

Simultaneous failure of both TPS tracks to the same incorrect value,
defeating the safety monitor's dual-track divergence check. This is the
common-cause failure mode requiring separate physical track routing.
