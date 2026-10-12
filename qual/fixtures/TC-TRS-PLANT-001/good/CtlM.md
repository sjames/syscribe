---
type: StateDef
name: CtlM
variables:
  - {name: u, value: "1"}
  - {name: y, value: "0"}
subStates:
  - name: Run
    isInitial: true
    transitions:
      - {target: Settled, accept: {when: "y > 0.99"}}
  - name: Settled
    transitions:
      - {target: Run, accept: {when: "y < 0.9"}}
---

Drives the lag with a unit step and notes when it has settled.
