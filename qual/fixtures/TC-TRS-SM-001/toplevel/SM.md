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
    accept:
      payload: Items::Cmd
---

Canonical top-level transitions: a `transitions:` list with explicit `source:`. Same machine
as the nested fixture; must produce the same edge model.
