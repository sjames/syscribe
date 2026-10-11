---
type: StateDef
name: Supervisor
variables:
  - {name: rIso, value: "5000"}
  - {name: gridOk, value: "0"}
subStates:
  - name: Standby
    isInitial: true
    transitions:
      - {target: WaitGrid, accept: Start, guard: "gridOk == 1 and rIso >= 34"}
      - {target: InsulationFault, accept: Start, guard: "gridOk == 1 and rIso < 34"}
      - {internal: true, accept: {payload: GridGood, via: grid}, assign: "gridOk := 1"}
      - {internal: true, accept: {payload: GridLost, via: grid}, assign: "gridOk := 0"}
  - name: InsulationFault
    transitions:
      - {target: WaitGrid, accept: Start, guard: "gridOk == 1 and rIso >= 34"}
      - {internal: true, accept: {payload: GridGood, via: grid}, assign: "gridOk := 1"}
      - {internal: true, accept: {payload: GridLost, via: grid}, assign: "gridOk := 0"}
  - name: WaitGrid
    transitions:
      - {target: Closing, accept: {after: "{{Features::GridProfile.reconnectDelayS}} s"}, effect: {send: Close, via: relayCmd}}
      - {target: Tripped, accept: {payload: GridLost, via: grid}, assign: "gridOk := 0", effect: {send: Shutdown, via: relayCmd}}
  - name: Closing
    transitions:
      - {target: Ramping, accept: {after: "100 ms"}, effect: {send: Enable, via: drive}}
      - {target: Tripped, accept: {payload: GridLost, via: grid}, assign: "gridOk := 0", effect: {send: Shutdown, via: relayCmd}}
  - name: Ramping
    transitions:
      - {target: Running, accept: {after: "5 s"}}
      - {target: Tripped, accept: {payload: GridLost, via: grid}, assign: "gridOk := 0", effect: {send: Shutdown, via: relayCmd}}
  - name: Running
    transitions:
      - {target: Tripped, accept: {payload: GridLost, via: grid}, assign: "gridOk := 0", effect: {send: Shutdown, via: relayCmd}}
  - name: Tripped
    entryAction: {send: Shutdown, via: drive}
    transitions:
      - {target: WaitGrid, accept: {payload: GridGood, via: grid}, assign: "gridOk := 1"}
---

Inverter connection supervisor (the reconnect delay is a parameter of the grid profile). Start is ignored until the monitor has reported the grid good (`gridOk`), then passes an insulation check (`rIso` in kΩ; the limit 34 kΩ is 1000 V / 30 mA, `[secondary]`, RM-16), waits the reconnect delay of the profile ({{Features::GridProfile.reconnectDelayS}} s, RM-07, **unverified**), closes both grid relays, enables the power stage and ramps for 5 s. A grid-loss report shuts everything down; the supervisor re-enters the wait only when the monitor reports the grid good again.
