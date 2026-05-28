---
type: Part
name: Watchdog Timer
typedBy: System::Hardware::WatchdogTimer
domain: hardware
---

External windowed watchdog IC (TPS382x-compatible) mounted on the Engine ECU PCB. The
external placement satisfies the ISO 26262-6 §9 independence requirement for the ASIL D
hardware channel — it operates from separate supply rails and has no software-accessible
configuration registers that could be corrupted by an MCU fault.

## Windowed mode operation

The watchdog must be serviced by the AUTOSAR OS on a 10 ms period — strictly within a
±2 ms window centred on the expected service time. Servicing too early (before the window
opens) or too late (after the window closes) both trigger a reset. This double-boundary
mode prevents a stuck loop from servicing the watchdog at a fixed offset without executing
the full OS tick path.

## Reset output connections

The reset pulse (active-low, 200 ms width) drives:
1. MCU nRST pin — causes a full MCU hardware reset.
2. Throttle actuator enable line — removes H-bridge power, allowing the return spring to
   close the throttle plate to the limp-home position within the 100 ms FTTI.

Both outputs are asserted simultaneously, ensuring that a software lock-up cannot hold the
throttle open while the MCU is being reset.

## Service obligation

The `SafetyMonitor` SWC is responsible for issuing the watchdog service call after completing
its 5 ms supervision cycle. If `SafetyMonitor` itself faults or is preempted beyond its
deadline, the watchdog resets the ECU within one missed service window (≤ 20 ms worst case).
