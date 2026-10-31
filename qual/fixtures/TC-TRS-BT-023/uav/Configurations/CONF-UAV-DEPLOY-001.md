---
type: Configuration
id: CONF-UAV-DEPLOY-001
name: "UAV — deployment settings"
status: approved
featureModel: Features
bindingStage: load
derivedFrom: CONF-UAV-BUILD-001
features:
  Features::Telemetry: true
parameterBindings:
  Features::Propulsion.motorKV: 1000
  Features::Propulsion.maxCurrentA: 45
  Features::Telemetry.mode: sport
---

Deployment: binds the motor constant, the current limit and the telemetry mode.
