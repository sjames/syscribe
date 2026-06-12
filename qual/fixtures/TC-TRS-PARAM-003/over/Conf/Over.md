---
type: Configuration
id: CONF-RANGE-OVER-001
name: "cores bound above the inclusive range"
status: approved
featureModel: Features
features:
  Features::Soc: true
parameterBindings:
  Features::Soc.cores: 99
---
cores = 99 is outside range 1..=8 → E205.
