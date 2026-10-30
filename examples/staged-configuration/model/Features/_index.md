---
type: Package
name: Features
parameterConstraints:
  - id: PC-UAV-CURRENT
    expression: "Features::Propulsion.motorKV * 0.04 + Features::Propulsion.maxCurrentA <= 100"
    appliesWhen: Features::Propulsion
    severity: error
---

The UAV feature model. Each feature and parameter says at which stage it is bound.
