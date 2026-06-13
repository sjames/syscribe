---
type: StateDef
name: Composite
subStates:
  - name: Outer
    typedBy: Parallel
    isInitial: true
    transitions:
      - target: Done
  - name: Done
    isFinal: true
---

A well-formed composite machine: `Outer` (typed by another StateDef) is the initial state
and transitions to a final `Done`. The flat W070–W074 must not fire. Composite interiors and
endpoint resolution are exercised by TC-TRS-SM-005.
