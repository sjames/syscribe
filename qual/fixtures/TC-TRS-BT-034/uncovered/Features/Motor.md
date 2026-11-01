---
type: FeatureDef
id: FEAT-FX-SF1
name: Motor
groupKind: optional
parameters:
  - name: kv
    type: ScalarValues::Real
    range: "900..1200"
    bindingTime: load
    latestBindingTime: runtime
  - name: old
    type: ScalarValues::Real
    range: "0..10"
    bindingTime: compile
---
A feature.
