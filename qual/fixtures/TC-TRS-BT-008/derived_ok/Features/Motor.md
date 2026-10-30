---
type: FeatureDef
id: FEAT-FX-B82
name: Motor
groupKind: optional
parameters:
  - name: a
    type: ScalarValues::Real
    range: "0..1"
    bindingTime: runtime
    derivedFrom: "b > 10"
  - name: b
    type: ScalarValues::Real
    range: "0..100"
    bindingTime: compile
    latestBindingTime: load
---
A fixture feature.
