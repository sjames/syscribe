---
type: StateDef
name: M
variables:
  - {name: a, value: "1"}
  - {name: b, value: "1"}
  - {name: c, value: "0"}
subStates:
  - name: Idle
    isInitial: true
    transitions:
      - {target: Slow, accept: Slow}
      - {target: B1, accept: Blip}
      - {target: Tripping, accept: Trip}
  - name: Slow
    transitions:
      - {target: Off, accept: {after: "2200 ms"}, assign: "a := 0; b := 0; c := 1"}
  - name: B1
    transitions:
      - {target: B2, accept: {after: "1 s"}, assign: "a := 0; b := 0"}
  - name: B2
    transitions:
      - {target: B3, accept: {after: "500 ms"}, assign: "a := 1"}
  - name: B3
    transitions:
      - {target: Off, accept: {after: "700 ms"}, assign: "a := 0"}
  - name: Tripping
    transitions:
      - {target: Off, accept: {after: "100 ms"}, assign: "a := 0; b := 0; c := 1"}
  - name: Off
---
