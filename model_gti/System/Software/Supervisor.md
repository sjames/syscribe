---
type: PartDef
name: Supervisor
domain: software
silLevel: 1
satisfies:
  - REQ-GTI-SYS-010
  - REQ-GTI-SYS-036
  - REQ-GTI-SYS-072
  - REQ-GTI-SYS-080
  - REQ-GTI-SW-007
  - REQ-GTI-SW-005
---

Inverter state machine: standby, insulation check, waiting for grid, ramp, run, derate, fault. Owns start/stop of the trackers and the relays; see Behavior::GridSupervisor.
