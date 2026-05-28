---
type: Part
name: Software Image
typedBy: System::EngineControlSoftware
domain: software
---

The deployed engine control software image, version-locked to the vehicle calibration
dataset. Contains all AUTOSAR SWC instances executing on the ECU microcontroller under
the AUTOSAR OS scheduler.

## Version locking

The software image version and calibration dataset version are bound together at the
ECU programming step. The `SecureBootManager` enforces this binding at every reset via the
ECDSA P-256 signature (REQ-ENG-SEC-003): the signed image digest covers both the application
binary and the calibration flash sector hash, so a version mismatch between code and
calibration fails the signature check and prevents execution.

## AUTOSAR OS configuration

The software image runs on a static schedule table:

| Task | Period | ASIL | SWC(s) |
|---|---|---|---|
| SafetyMonitorTask | 5 ms | D | SafetyMonitor |
| ThrottleControlTask | 10 ms | QM | ThrottleControl |
| FuelControlTask | 10 ms | QM | FuelControl |
| StallMonitorTask | 20 ms | B | EngineStallMonitor |
| CANSecTask | event | QM | CANSecurityModule |
| BootTask | once | QM | BootSequence |

The ASIL D task is non-preemptable and has an MPU region grant that QM tasks do not.
No dynamic task creation is used; the schedule table is fixed at compile time.
