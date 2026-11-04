---
type: Package
name: Features
parameterConstraints:
  - id: PC-PLF-BUFFER
    expression: "Features::Logging.bufferKiB * Features::Logging.rateHz <= 4096"
    appliesWhen: Features::Logging
    severity: error
  - id: PC-PLF-KV
    expression: "Features::Propulsion.motorKV <= 1200"
    appliesWhen: Features::Propulsion
    severity: warning
---

The feature model of the fixture.
