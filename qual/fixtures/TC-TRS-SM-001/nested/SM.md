---
type: StateDef
name: SM
subStates:
  - name: A
    isInitial: true
    transitions:
      - target: B
        accept: Items::Cmd
  - name: B
    transitions:
      - target: A
        accept:
          payload: Items::Cmd
---

Canonical nested transitions: per-substate `transitions:` with implicit source, mixing the
`accept:` string shorthand and the `{payload:}` map form.
