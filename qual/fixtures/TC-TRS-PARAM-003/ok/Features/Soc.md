---
type: FeatureDef
id: FEAT-FX-086
name: Soc
groupKind: optional
parameters:
  - name: cores
    type: ScalarValues::Integer
    range: "1..=8"
---
SoC with a core count constrained to the inclusive range 1..=8.
