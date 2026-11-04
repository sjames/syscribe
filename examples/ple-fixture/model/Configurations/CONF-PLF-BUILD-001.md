---
type: Configuration
id: CONF-PLF-BUILD-001
name: "Fixture — build"
status: approved
featureModel: Features
bindingStage: compile
features:
  Features::Platform::CortexM: true
  Features::Link::Lora: true
  Features::Propulsion: true
parameterNarrowing:
  Features::Propulsion.motorKV: "900..1100"
---

Build time: the platform, a link and the propulsion family; narrows the motor constant.
