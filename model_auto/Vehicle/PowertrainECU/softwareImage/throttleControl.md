---
type: Part
name: Throttle Control
typedBy: System::Software::ThrottleControl
domain: software
---

ThrottleControl AUTOSAR SWC instance running in the 10 ms ThrottleControlTask (QM partition).
Implements the closed-loop PID position controller between the demanded throttle angle
(from driver pedal and cruise control arbitration) and the measured TPS track-1 position.

## PID configuration

- Proportional gain (Kp): 2.5 (calibrated for 60 mm bore throttle body dynamics)
- Integral gain (Ki): 0.8 s⁻¹
- Derivative gain (Kd): 0.05 s
- Control cycle: 10 ms; output clamped to −100 %…+100 % H-bridge duty cycle
- Anti-windup: integrator clamped when output is saturated (back-calculation method)

## Fail-safe and limp-home

When `SafetyMonitor` asserts the fault signal, `ThrottleControl` immediately sets output
to 0 % duty cycle (H-bridge open, spring takes over) within the same 10 ms cycle. The
limp-home mode sets a fixed 7 % demand, bypassing the PID loop, to allow the vehicle to
be driven to a service location at idle speed.

## CAN output

The throttle control command is published to the powertrain CAN bus via `canOut` port
(arbitration ID 0x0C8, 10 ms cycle, 8-byte payload). The `CANSecurityModule` appends
a 24-bit CMAC-AES-128 MAC to the frame before transmission.
