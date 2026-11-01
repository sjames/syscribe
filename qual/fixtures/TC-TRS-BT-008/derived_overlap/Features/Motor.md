---
type: FeatureDef
id: FEAT-FX-B83
name: Motor
groupKind: optional
parameters:
  - name: a
    type: ScalarValues::Real
    range: "0..1"
    bindingTime: compile
    latestBindingTime: load
    derivedFrom: "b > 10"
  - name: b
    type: ScalarValues::Real
    range: "0..100"
    bindingTime: load
---
A fixture feature.
