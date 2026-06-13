---
type: StateDef
name: SM
status: draft
isParallel: true
subStates:
  - name: OnlyRegion
    subStates:
      - name: S1
---

Single-region parallel state with a malformed region, but `status: draft` — W078 and the
region's W073/W070/W071 are all suppressed.
