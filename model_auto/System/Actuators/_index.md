---
type: Package
name: Actuators
---

Output actuator PartDefs driven by the Engine ECU. Actuators represent the physical
effectors controlled by the software SWCs in `System/Software/`.

## Actuator inventory

| PartDef | Drive circuit | Fail-safe state | Safety link |
|---|---|---|---|
| `ThrottleActuator` | H-bridge (DC motor), PWM | Spring-return to ~7 % opening | SG-ENG-001, REQ-ENG-SAFE-001/005 |
| `FuelInjector` | Peak-and-hold solenoid (4 A → 1 A) | Off (no injection pulse) | SG-ENG-003, REQ-ENG-SAFE-004 |

## Fail-safe design

**Throttle actuator** — The return spring provides a passive fail-safe: loss of drive current
returns the throttle plate to approximately 7 % opening, providing enough airflow for the
engine to idle but preventing uncontrolled acceleration. The `SafetyMonitor` can assert a
fault signal that causes `ThrottleControl` to cease commanding the H-bridge, allowing the
spring to take over within the FTTI window.

**Fuel injectors** — A fuel cut is implemented by withholding injection pulses entirely.
The rev limiter (soft ignition retard at 6200 rpm, hard fuel cut at 6500 rpm) uses this
mechanism. The `SafetyMonitor` can also command a full fuel cut independent of the `FuelControl`
PID loop, providing an independent shut-off path for over-speed or fault conditions.
