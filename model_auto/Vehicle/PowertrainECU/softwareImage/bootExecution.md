---
type: Action
name: Boot Execution
typedBy: System::Software::BootSequence
---

Boot sequence action executed on the PowertrainECU hardware context at every reset event.
The full boot sequence must complete within 200 ms before any engine control loop is enabled.
This budget is allocated as follows:

| Stage | Owner | Budget |
|---|---|---|
| Hardware self-test (RAM, ROM, ADC) | BSW StartupHook | 20 ms |
| Watchdog initialisation (windowed mode enable) | AUTOSAR WdgM | 5 ms |
| SecureBootManager signature verification | SecureBootManager | 80 ms |
| SecOC key load from HSM | CANSecurityModule | 30 ms |
| Sensor plausibility (TPS track 1/2 within 5 %) | SafetyMonitor | 10 ms |
| SafetyMonitor activation (first supervision cycle) | SafetyMonitor | 5 ms |
| Throttle actuator enable | SafetyMonitor → H-bridge | 5 ms |

## Timing constraints

The 200 ms total budget is driven by the FTTI for SG-ENG-001 (100 ms) and by the requirement
that the actuator enable lockout covers the SecOC startup window (VR-ENG-001 mitigation).
The actuator enable line must remain low until SecOC keys are loaded (stage 4 complete),
ensuring no throttle command can be acted upon before MAC authentication is functional.

## Failure modes

If `SecureBootManager` signature verification fails, the ECU halts and sets DTC U3001
(internal ECU fault). If sensor plausibility fails (TPS divergence > 5 % at rest),
`SafetyMonitor` logs the fault and the ECU enters limp-home mode without activating the
throttle actuator. Both failure modes require a key-off/on cycle to re-attempt boot.
