---
type: Configuration
id: CONF-PLF-DEPLOY-001
name: "Fixture — deployment"
status: approved
featureModel: Features
bindingStage: load
derivedFrom: CONF-PLF-BUILD-001
features:
  Features::Logging: true
parameterBindings:
  Features::Propulsion.motorKV: 1000
  Features::Logging.bufferKiB: 8
  Features::Logging.levelName: info
---

Deployment: binds the motor constant and the logging buffer.
