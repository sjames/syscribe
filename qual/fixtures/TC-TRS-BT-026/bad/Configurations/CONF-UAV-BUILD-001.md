---
type: Configuration
id: CONF-UAV-BUILD-001
name: "UAV — build settings"
status: approved
featureModel: Features
bindingStage: compile
features:
  Features::Propulsion: true
parameterNarrowing:
  Features::Propulsion.motorKV: "900..1100"
---

Build time: fixes the propulsion family and narrows the motor constant to what this
airframe can take, leaving the exact value to deployment.
