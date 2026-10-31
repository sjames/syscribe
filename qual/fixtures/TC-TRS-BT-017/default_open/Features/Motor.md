---
type: FeatureDef
id: FEAT-FX-RS1
name: Motor
groupKind: optional
parameters:
  - name: kv
    type: ScalarValues::Real
    range: "900..1200"
    bindingTime: compile
    latestBindingTime: load
  - name: old
    type: ScalarValues::Real
    range: "0..100"
    default: 90
    bindingTime: compile
    latestBindingTime: load
  - name: dflt
    type: ScalarValues::Real
    range: "0..100"
    default: 90
    bindingTime: load
    latestBindingTime: runtime
---
A fixture feature.
