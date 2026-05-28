---
type: Package
name: Allocations
---

This package contains allocation elements that map software components to hardware
deployment targets. In AUTOSAR terms each allocation corresponds to an ECU extract
pairing an SWC cluster with a specific ECU instance and its BSW configuration.

## Contents

| Element | From (software) | To (hardware) |
|---|---|---|
| `SwToECU` | `System::EngineControlSoftware` | `System::EngineECU` |

## Freedom from interference

The allocation document (`SwToECU`) records the memory protection and temporal isolation
measures required by ISO 26262-6 §7.4.5 freedom-from-interference:

- **Spatial isolation** — AUTOSAR MemoryProtection (MPU) partitions ASIL D code and data
  from QM regions. Each ASIL partition occupies a dedicated MPU region with no overlap.
- **Temporal isolation** — AUTOSAR OS schedule table reserves a fixed time budget per OS
  task. ASIL D tasks (SafetyMonitor, 5 ms) are non-preemptable by QM tasks.
- **Data consistency** — Shared inter-SWC data is exchanged via AUTOSAR RTE ports; direct
  global variable access across ASIL boundaries is prohibited.

The allocation also specifies which calibration data sectors in flash are write-protected
during normal operation and unlocked only under security access level 0x27/0x28.
