---
type: StateDef
name: CtlM
subStates:
  - name: Idle
    isInitial: true
    transitions:
      - {target: Run, accept: Go}
  - name: Run
    transitions:
      - {target: Idle, accept: Stop}
      - {target: Fault, accept: Bad}
  - name: Fault
---
