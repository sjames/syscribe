---
type: FeatureDef
id: FEAT-FX-027
name: Cyc
groupKind: optional
parameters:
  - name: p1
    type: ScalarValues::Real
    derivedFrom: "p2 + 1"
  - name: p2
    type: ScalarValues::Real
    derivedFrom: "p1 + 1"
---
Cyclic derivation.
