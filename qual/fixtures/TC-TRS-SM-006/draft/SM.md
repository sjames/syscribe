---
type: StateDef
name: SM
status: draft
subStates:
  - name: Init
    isInitial: true
    transitions:
      - target: Done
        effect: Behavior::NoSuchAction
  - name: Done
    isFinal: true
---

Unresolved `effect` reference, but `status: draft` — W079 suppressed.
