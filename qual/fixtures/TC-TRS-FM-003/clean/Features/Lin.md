---
type: FeatureDef
id: FEAT-FX-024
name: Lin
groupKind: optional
parameters:
  - name: base
    type: ScalarValues::Real
  - name: scaled
    type: ScalarValues::Real
    derivedFrom: "base * 2"
---
Acyclic derivation.
