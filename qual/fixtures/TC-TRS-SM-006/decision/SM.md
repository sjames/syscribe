---
type: StateDef
name: SM
subStates:
  - name: Init
    isInitial: true
    transitions:
      - target: A
        accept: Items::Cmd
        guard: "x > 0"
      - target: B
        accept: Items::Cmd
        guard: "x <= 0"
  - name: A
    isFinal: true
  - name: B
    isFinal: true
---

A decision transition: two transitions from `Init` accept the same payload but are
distinguished by guards. This is legitimate branching — W072 must NOT fire.
