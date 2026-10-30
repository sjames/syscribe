---
type: Package
name: Features
parameterConstraints:
  - id: PC-BT-1
    expression: "Features::Motor.kv + Features::Motor.spin <= 1100"
    appliesWhen: Features::Motor
    severity: error
---
Features with one cross-parameter constraint.
