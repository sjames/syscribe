---
type: FeatureDef
id: FEAT-LIMITS
name: Limits
mandatory: true
parameters:
  - { name: maxTemp, type: ScalarValues::Integer, default: 120, unit: degC, range: "0..200" }
---

The limits the heater is held to.
