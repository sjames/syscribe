---
type: StateDef
name: SM
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

Legacy transition spelling using `from`/`to`/`trigger` — must raise W075 while still forming
the connected A↔B machine.
