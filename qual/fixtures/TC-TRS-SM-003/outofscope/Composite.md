---
type: StateDef
name: Composite
subStates:
  - name: Outer
    typedBy: Parallel
    isInitial: true
  - name: Other
---

A composite machine: the `Outer` substate is typed by another StateDef. Out of scope for the
flat checks; must raise none of W070–W074 (handled by the hierarchy-aware phase).
