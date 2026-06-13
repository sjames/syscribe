---
type: StateDef
name: SM
isParallel: true
subStates:
  - name: RA
    subStates:
      - name: A1
        isInitial: true
        transitions:
          - target: A2
            accept: Items::E
      - name: A2
        isFinal: true
  - name: RB
    subStates:
      - name: B1
        isInitial: true
        isFinal: true
---

Well-formed parallel machine: two regions, each with exactly one initial and a connected /
terminating path. No W070–W074, W077, W078.
