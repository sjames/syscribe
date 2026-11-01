---
type: FeatureDef
id: FEAT-FX-B81
name: Motor
groupKind: optional
parameters:
  - name: a
    type: ScalarValues::Real
    range: "0..1"
    bindingTime: compile
    derivedFrom: "b > 10"
  - name: b
    type: ScalarValues::Real
    range: "0..100"
    bindingTime: load
    latestBindingTime: runtime
---
A fixture feature.
