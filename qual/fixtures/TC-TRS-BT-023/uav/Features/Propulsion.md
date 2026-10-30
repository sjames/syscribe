---
type: FeatureDef
id: FEAT-UAV-PROP
name: Propulsion
mandatory: true
groupKind: mandatory
parameters:
  - name: motorKV
    type: ScalarValues::Real
    unit: rpm/V
    range: "900..1200"
    isRequired: true
    bindingTime: compile
    latestBindingTime: load
  - name: maxCurrentA
    type: ScalarValues::Real
    unit: A
    range: "0..60"
    default: 40
    fallback: 30
    bindingTime: load
    latestBindingTime: runtime
---

Propulsion. The motor constant is fixed at build or at deployment; the current limit
can still change in the field.
