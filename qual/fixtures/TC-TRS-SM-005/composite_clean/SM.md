---
type: StateDef
name: SM
subStates:
  - name: Active
    isInitial: true
    subStates:
      - name: Warmup
        isInitial: true
        transitions:
          - target: Running
      - name: Running
        isFinal: true
    transitions:
      - target: Done
  - name: Done
    isFinal: true
---

Well-formed composite machine. Top level: `Active` (composite, initial) → `Done` (final).
Inner region of `Active`: `Warmup` (initial) → `Running` (final). None of W070–W078.
