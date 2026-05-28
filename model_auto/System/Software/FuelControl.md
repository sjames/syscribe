---
type: PartDef
name: Fuel Control
domain: software
satisfies:
  - REQ-ENG-PERF-002
features:
  - name: injectionMode
    type: ScalarValues::String
  - name: lambdaTarget
    type: ScalarValues::Real
---

Closed-loop fuel injection control component. Computes injection pulse width
from engine speed, load, and lambda feedback to maintain stoichiometric mixture.

## Fuel calculation

Base fuel mass is computed from a 3-D calibration map (speed × load). Lambda
correction is applied via a PI controller using the wideband lambda sensor output.
Cold-start enrichment is applied for coolant temperature below 60 °C.
