---
type: StateDef
name: Parallel
isParallel: true
subStates:
  - name: RegionA
    subStates:
      - name: A1
  - name: RegionB
    subStates:
      - name: B1
---

A parallel (orthogonal) state machine — out of scope for the flat W070–W074 checks; must
raise none of them (handled by the region-aware phase).
