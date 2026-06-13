---
type: StateDef
name: Parallel
isParallel: true
subStates:
  - name: RegionA
    subStates:
      - name: A1
        isInitial: true
        isFinal: true
  - name: RegionB
    subStates:
      - name: B1
        isInitial: true
        isFinal: true
---

A well-formed parallel (orthogonal) state machine — two regions, each with a single
initial/final state. The flat W070–W074 must not fire (it is region-aware; this machine is
clean). Region/cross-region defects are exercised by TC-TRS-SM-004.
