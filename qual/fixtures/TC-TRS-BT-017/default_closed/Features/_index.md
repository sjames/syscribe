---
type: Package
name: Features
parameterConstraints:
  - id: PC-BT-1
    expression: "Features::Motor.old + 10 <= 95"
    appliesWhen: Features::Motor
    severity: error
---
Features with one cross-parameter constraint.
