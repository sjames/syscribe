---
type: PartDef
name: Throttle Control
domain: software
satisfies:
  - REQ-ENG-PERF-001
features:
  - name: controlCycleMs
    type: ScalarValues::Integer
    unit: ms
  - name: pidKp
    type: ScalarValues::Real
  - name: pidKi
    type: ScalarValues::Real
---

Electronic throttle control (ETC) software component. Implements a PID
position controller converting pedal demand (from TPS) to a throttle plate
position command.

## Control law

Output is clamped to 0–100 % opening. The controller runs at 10 ms cycle
rate to meet the 50 ms response latency requirement (REQ-ENG-PERF-001).

Anti-windup is active when the actuator is at its mechanical limits. In any
detected fault state (from SafetyMonitor), the output is frozen at the
fail-safe value and a DTC is set.
