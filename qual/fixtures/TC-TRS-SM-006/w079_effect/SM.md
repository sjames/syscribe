---
type: StateDef
name: SM
subStates:
  - name: Init
    isInitial: true
    transitions:
      - target: Done
        effect: Behavior::NoSuchAction
  - name: Done
    isFinal: true
---

The `Init` → `Done` transition's `effect:` names `Behavior::NoSuchAction`, which resolves to
no model element — W079. The machine is otherwise well-formed.
