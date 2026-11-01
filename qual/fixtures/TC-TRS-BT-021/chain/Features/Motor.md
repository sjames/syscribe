---
type: FeatureDef
id: FEAT-FX-PV1
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
  - name: lim
    type: ScalarValues::Real
    value: 5
  - name: dflt
    type: ScalarValues::Real
    range: "0..10"
    default: 3
  - name: ratio
    type: ScalarValues::Real
    derivedFrom: "kv * 2"
---
Motor with a parameter of every value source.
