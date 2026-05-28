---
type: ActionDef
name: Boot Sequence
domain: software
---

Defines the ordered startup sequence executed by the Engine Control Software
on every microcontroller reset. Actions within this definition are performed
in succession before any closed-loop engine control begins.

## Sequence

1. Hardware self-test (RAM, ROM CRC, supply voltage)
2. Watchdog initialisation (windowed mode, 10 ms window)
3. SecOC key loading and MAC initialisation
4. Sensor plausibility check (TPS track divergence, CPS signal present)
5. Safety monitor activation
6. Throttle actuator enable (released only after all checks pass)
