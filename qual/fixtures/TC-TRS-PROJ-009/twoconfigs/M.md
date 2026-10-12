---
type: StateDef
name: M
variables:
  - {name: level, value: "0"}
subStates:
  - name: Wait
    isInitial: true
    transitions:
      - {target: Armed, accept: {after: "{{Features::Profile.delayS}} s"}, assign: "level := {{Features::Profile.limit}} + 1"}
  - name: Armed
    transitions:
      - {target: Over, accept: Check, guard: "level > {{Features::Profile.limit}}"}
  - {name: Over, isFinal: true}
---
