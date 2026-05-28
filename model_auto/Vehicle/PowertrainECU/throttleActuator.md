---
type: Part
name: Throttle Actuator
typedBy: System::Actuators::ThrottleActuator
domain: hardware
---

Electronic throttle body (ETB) instance with a 60 mm bore aluminium housing. A DC motor
drives the throttle plate via a reduction gear train (ratio ≈ 1:50); a coil spring provides
the fail-safe return force.

## Fail-safe spring

The return spring is rated for 8 N closing force and returns the throttle plate to the
limp-home position (approximately 7 % opening — sufficient for engine idle) whenever motor
drive current is removed. This is the passive safety mechanism for SG-ENG-001: if the ECU
loses power, the MCU asserts a fault, or the `SafetyMonitor` inhibits the H-bridge driver,
the spring closes the throttle within the 100 ms FTTI.

## H-bridge drive

The motor is driven by an H-bridge power stage on the ECU PCB, controlled by two PWM
signals from the MCU. Direction and duty cycle determine position. The `ThrottleControl`
SWC PID output (0–100 %) maps linearly to H-bridge duty cycle after dead-band compensation.

## Integrated position feedback

The throttle body contains the `tpsSensor` (dual-track TPS) as a co-located sub-assembly.
The mechanical coupling between the throttle shaft and TPS wiper arm is rigid; any gear
or shaft play larger than ±0.5° manifests as track-1/track-2 correlation error.
