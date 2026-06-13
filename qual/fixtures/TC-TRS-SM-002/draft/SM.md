---
type: StateDef
name: SM
status: draft
subStates:
  - name: A
    isInitial: true
  - name: B
transitions:
  - from: A
    to: B
    trigger: Items::Cmd
  - from: B
    to: A
    trigger: Items::Cmd
---

Legacy spelling but `status: draft` — W075 must be suppressed.
