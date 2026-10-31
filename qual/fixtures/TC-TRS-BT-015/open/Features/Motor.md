---
type: FeatureDef
id: FEAT-FX-ST1
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
  - name: mode
    type: ScalarValues::String
    enumValues: [eco, sport, race]
    bindingTime: compile
    latestBindingTime: load
  - name: reqearly
    type: ScalarValues::Real
    range: "0..10"
    isRequired: true
    bindingTime: compile
  - name: reqlate
    type: ScalarValues::Real
    range: "0..10"
    isRequired: true
    bindingTime: load
    latestBindingTime: runtime
---
Motor with parameters at different binding windows.
