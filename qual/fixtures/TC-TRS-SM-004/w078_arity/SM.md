---
type: StateDef
name: SM
isParallel: true
subStates:
  - name: OnlyRegion
    subStates:
      - name: S1
        isInitial: true
        isFinal: true
---

A parallel state with a single region — W078. The lone region is itself well-formed.
