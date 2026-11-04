---
id: FEAT-PLF-PROP
type: FeatureDef
name: Propulsion
mandatory: true
parameters:
  - name: motorKV
    type: ScalarValues::Real
    unit: rpm/V
    range: "900..1200"
    isRequired: true
    bindingTime: compile
    latestBindingTime: load
---

Propulsion; the motor constant is bound at build or deployment.
