---
type: FeatureDef
id: FEAT-FX-B73
name: Motor
groupKind: optional
bindingTime: compile
latestBindingTime: load
parameters:
  - name: kv
    type: ScalarValues::Real
    range: "0..100"
    bindingTime: compile
    latestBindingTime: runtime
---
A fixture feature.
