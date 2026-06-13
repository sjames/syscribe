---
type: StateDef
name: SM
isParallel: true
subStates:
  - name: RA
    subStates:
      - name: A1
        isInitial: true
        isFinal: true
  - name: RB
    subStates:
      - name: B1
        isInitial: true
        isFinal: true
transitions:
  - source: A1
    target: B1
---

A top-level transition `A1` → `B1` connects substates in two different regions — W077.
Each region is otherwise well-formed.
