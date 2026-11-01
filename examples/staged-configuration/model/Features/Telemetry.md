---
type: FeatureDef
id: FEAT-UAV-TELEM
name: Telemetry
groupKind: optional
bindingTime: load
latestBindingTime: runtime
parameters:
  - name: rateHz
    type: ScalarValues::Real
    unit: Hz
    range: "1..50"
    default: 10
    fallback: 5
    bindingTime: runtime
  - name: mode
    type: ScalarValues::String
    enumValues: [eco, normal, sport]
    isRequired: true
    fallback: normal
    bindingTime: load
    latestBindingTime: runtime
---

Telemetry downlink. Whether it is fitted is decided at deployment or later; its rate is
chosen by the running system.
