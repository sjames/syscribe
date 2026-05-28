---
type: PartDef
name: Watchdog Timer
domain: hardware
asilLevel: D
satisfies:
  - REQ-ENG-SAFE-002
features:
  - name: timeoutMs
    type: ScalarValues::Integer
    unit: ms
  - name: windowedMode
    type: ScalarValues::Boolean
---

External windowed hardware watchdog timer (ASIL D). The SafetyMonitor software
component must service the watchdog within the window every 10 ms. Failure to
service — caused by software lock-up, stack overflow, or runaway code — triggers
a hardware reset of the microcontroller.

Windowed mode rejects early servicing, preventing a stuck-loop false-service
scenario. Reset output is connected to the MCU reset pin and the throttle
actuator enable line.
