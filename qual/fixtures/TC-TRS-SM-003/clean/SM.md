---
type: StateDef
name: SM
subStates:
  - name: Init
    isInitial: true
    transitions:
      - target: Run
        accept: Items::Start
  - name: Run
    transitions:
      - target: Done
        accept: Items::Stop
  - name: Done
    isFinal: true
---

Well-formed single-region machine: one initial, connected, terminating in a final state.
None of W070–W074.
