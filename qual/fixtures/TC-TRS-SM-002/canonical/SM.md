---
type: StateDef
name: SM
subStates:
  - name: A
    isInitial: true
  - name: B
transitions:
  - source: A
    target: B
    accept: Items::Cmd
  - source: B
    target: A
    accept: Items::Cmd
---

Canonical spelling of the same machine — must raise no W075.
