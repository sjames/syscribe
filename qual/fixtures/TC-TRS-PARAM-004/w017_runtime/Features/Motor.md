---
type: FeatureDef
id: FEAT-FX-093
name: Motor
groupKind: optional
parameters:
  - name: gain
    type: ScalarValues::Real
    range: "0..10"
    isRequired: true
    bindingTime: runtime
---
Motor feature with a required runtime parameter.
