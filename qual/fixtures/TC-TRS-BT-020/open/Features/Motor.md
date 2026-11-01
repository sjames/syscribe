---
type: FeatureDef
id: FEAT-FX-WS1
name: Motor
groupKind: optional
parameters:
  - name: reqcomp
    type: ScalarValues::Real
    range: "0..10"
    isRequired: true
    bindingTime: compile
  - name: reqload
    type: ScalarValues::Real
    range: "0..10"
    isRequired: true
    bindingTime: load
    latestBindingTime: runtime
  - name: reqrun
    type: ScalarValues::Real
    range: "0..10"
    isRequired: true
    bindingTime: runtime
---
A fixture feature.
