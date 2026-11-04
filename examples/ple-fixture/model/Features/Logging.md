---
id: FEAT-PLF-LOGGING
type: FeatureDef
name: Logging
bindingTime: load
latestBindingTime: runtime
parameters:
  - name: levelName
    type: ScalarValues::String
    enumValues: [error, warn, info]
    default: warn
    bindingTime: load
    latestBindingTime: runtime
  - name: bufferKiB
    type: ScalarValues::Real
    unit: KiB
    range: "1..64"
    default: 8
    bindingTime: load
    latestBindingTime: load
  - name: rateHz
    type: ScalarValues::Real
    unit: Hz
    range: "1..100"
    default: 10
    bindingTime: runtime
    latestBindingTime: runtime
---

Flight logging.
