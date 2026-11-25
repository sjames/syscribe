---
type: FeatureDef
id: FEAT-HEATER
name: Heater
mandatory: true
parameters:
  - { name: onTimeMs, type: ScalarValues::Integer, unit: ms, derivedFrom: "Features::Limits.maxTemp * 2" }
---

The heater element; its on time follows from the temperature limit.
