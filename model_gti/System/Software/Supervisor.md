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
features:
  - {name: profile, type: Port, direction: in}
  - {name: trip, type: Port, direction: in}
  - {name: insulation, type: Port, direction: in}
  - {name: selfTestFail, type: Port, direction: in}
  - {name: cmdIn, type: Port, direction: in}
  - {name: mpptCtl, type: Port, direction: out}
  - {name: gridCtl, type: Port, direction: out}
  - {name: event, type: Port, direction: out}
  - {name: relayCmd, type: Port, direction: out}
---

Inverter state machine: standby, insulation check, waiting for grid, ramp, run, derate, fault. Owns start/stop of the trackers and the relays; see Behavior::GridSupervisor.
