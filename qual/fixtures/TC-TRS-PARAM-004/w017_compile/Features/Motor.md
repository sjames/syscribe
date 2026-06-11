---
type: FeatureDef
id: FEAT-FX-092
name: Motor
groupKind: optional
parameters:
  - name: clk
    type: ScalarValues::Real
    range: "0..100"
    isRequired: true
    bindingTime: compile
---
Motor feature with a required compile-time parameter.
