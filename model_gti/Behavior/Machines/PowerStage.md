---
type: StateDef
name: PowerStage
variables:
  - {name: pwm, value: "0"}
subStates:
  - name: Off
    isInitial: true
    transitions:
      - {target: On, accept: {payload: Enable, via: cmd}, assign: "pwm := 1"}
  - name: On
    transitions:
      - {target: Off, accept: {payload: Shutdown, via: cmd}, assign: "pwm := 0"}
      - {target: Off, accept: {payload: Trip, via: hw}, assign: "pwm := 0"}
---

Gate-drive state of the bridge: `pwm` is 1 while the bridge is switching. A hardware trip turns the gates off independently of the firmware.
