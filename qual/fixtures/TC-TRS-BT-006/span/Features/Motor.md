---
type: FeatureDef
id: FEAT-FX-B61
name: Motor
groupKind: optional
bindingTime: compile
latestBindingTime: load
parameters:
  - name: kv
    type: ScalarValues::Real
    range: "0..100"
    bindingTime: compile
    latestBindingTime: load
  - name: lim
    type: ScalarValues::Real
    range: "0..100"
    latestBindingTime: load
  - name: one
    type: ScalarValues::Real
    range: "0..100"
    bindingTime: runtime
---
A fixture feature.
