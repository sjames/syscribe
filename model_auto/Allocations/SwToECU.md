---
type: Allocation
name: Software to ECU Hardware Allocation
allocatedFrom:
  - System::EngineControlSoftware
allocatedTo:
  - System::EngineECU
---

The Engine Control Software (`System::EngineControlSoftware`, `domain: software`,
`isDeploymentPackage: true`) is deployed to and executes on the Engine ECU
hardware (`System::EngineECU`, `domain: hardware`).

## Freedom from interference

The following mechanisms ensure freedom from interference between ASIL D and
QM software partitions (ISO 26262-6 §7.4.9):

- **Memory protection unit (MPU)** — SafetyMonitor and ThrottleControl SWCs
  have non-overlapping MPU regions
- **OS task isolation** — Safety-critical tasks run in a protected OS mode with
  separate stacks
- **Timing protection** — AUTOSAR OS timing protection prevents budget overruns
  from lower-ASIL tasks

## Calibration data

Calibration data (fuel maps, PID parameters) is stored in a separate flash
partition with write-protection hardware locks. Update requires a valid UDS
programming session (controlled by SC-ENG-002).
