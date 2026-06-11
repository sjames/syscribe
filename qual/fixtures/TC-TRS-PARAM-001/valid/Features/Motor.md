---
type: FeatureDef
id: FEAT-FX-076
name: Motor
groupKind: optional
parameters:
  - name: kv
    type: ScalarValues::Real
    range: "900..1200"
    isRequired: true
  - name: poles
    type: ScalarValues::Integer
    value: 14
    isFixed: true
  - name: esc
    type: ScalarValues::String
    enumValues: [dshot, pwm]
---
Motor feature.
