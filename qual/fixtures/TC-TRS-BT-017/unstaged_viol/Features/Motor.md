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
  - name: spin
    type: ScalarValues::Real
    range: "0..100"
    bindingTime: load
    latestBindingTime: runtime
  - name: gain
    type: ScalarValues::Real
    range: "0..10"
    bindingTime: runtime
  - name: lim
    type: ScalarValues::Real
    value: 5
  - name: cc
    type: ScalarValues::Real
    range: "0..10"
    bindingTime: compile
---
A fixture feature.
