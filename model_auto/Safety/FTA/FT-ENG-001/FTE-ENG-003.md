---
type: FaultTreeEvent
id: FTE-ENG-003
title: Throttle actuator mechanically stuck open above 20 %
eventKind: basic
ref: System::Actuators::ThrottleActuator
failureRate: 2.4e-7
probability: 2.4e-4
---

Mechanical jamming of the throttle plate in an open position exceeding
the fail-safe 7 % return-spring position. This defeats the software
fail-safe path; the hardware watchdog reset is the remaining protection.
