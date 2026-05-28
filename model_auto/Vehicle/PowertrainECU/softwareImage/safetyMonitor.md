---
type: Part
name: Safety Monitor
typedBy: System::Software::SafetyMonitor
domain: software
---

SafetyMonitor AUTOSAR SWC instance executing in the 5 ms ASIL D OS task with MPU-protected
stack and data regions. The monitor is the supervisory authority for all safety-relevant
I/O on the ECU; no actuator command reaches the hardware without passing through it.

## Supervision checks per 5 ms cycle

1. **TPS dual-track divergence** — compares track 1 and track 2; divergence > 5 % flags
   `FAULT_TPS_DIVERGE` and commands throttle to 0 % duty cycle (spring close).
2. **Throttle close verification** — if a close command has been active for > 200 ms and
   TPS reports > 10 % opening, flags `FAULT_THROTTLE_STUCK` and sets DTC P2111.
3. **Pedal vs TPS plausibility** — if accelerator pedal signal < 5 % but TPS > 20 %,
   flags `FAULT_TPS_PEDAL_MISMATCH`.
4. **Watchdog service** — issues the windowed watchdog service call after completing all
   checks. If any check blocks and the service call is not issued within the window, the
   external watchdog resets the ECU.
5. **Injector drive monitor** — checks drive-circuit current waveform presence; missing
   peak current on any channel sets the corresponding DTC (P0201–P0204).

## Fault response

On `FAULT_TPS_DIVERGE` or `FAULT_THROTTLE_STUCK`: throttle H-bridge disabled, fuel cut
asserted, DTC written, and the ECU enters limp-home mode. Recovery requires a key-off/on
cycle (fault latch cleared by reset, not by fault disappearance).
