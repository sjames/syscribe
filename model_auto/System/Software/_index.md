---
type: Package
name: Software
---

Software functional components (AUTOSAR SWCs) executing on the Engine ECU microcontroller.
Each PartDef in this package corresponds to one AUTOSAR application-layer SWC and is
allocated to the ECU via the `Allocations/SwToECU` element.

## Component inventory

| PartDef | ASIL | Cycle time | Key function |
|---|---|---|---|
| `SafetyMonitor` | D | 5 ms | Supervises all safety-relevant I/O; asserts fault signal |
| `ThrottleControl` | QM | 10 ms | PID position controller with anti-windup and limp-home |
| `FuelControl` | QM | 10 ms | Lambda closed-loop, rev limiter, cold-start enrichment |
| `EngineStallMonitor` | B | 20 ms | CPS tooth-count plausibility; DTC on 3 consecutive errors |
| `CANSecurityModule` | QM | event-driven | AUTOSAR SecOC MAC verification and generation |
| `DiagnosticSecurityLayer` | QM | on-demand | UDS session management and security access |
| `SecureBootManager` | QM | boot-time only | ECDSA P-256 firmware signature verification |
| `BootSequence` | — | once at reset | Ordered startup procedure (ActionDef) |

## ASIL partitioning

ISO 26262-6 §7.4.5 freedom-from-interference requires spatial and temporal isolation between
ASIL D and QM software. `SafetyMonitor` runs in a dedicated ASIL D OS task with MPU-protected
stack and data regions. QM SWCs are prevented from writing to ASIL D memory regions or
preempting the ASIL D task. See `Allocations/SwToECU` for the MPU map and OS schedule table.

## Startup ordering

The `BootSequence` ActionDef defines the mandatory startup ordering:
hardware self-test → watchdog initialisation → SecOC key load →
sensor plausibility check → safety monitor activation → throttle actuator enable.
No engine control loop is permitted to run until `SafetyMonitor` confirms readiness.
