---
type: Package
name: Configs
parameterConstraints:
  - id: PC-AMP-MIN
    expression: "Features::Topology.maxCpus >= 2"
    appliesWhen: Features::Topology::Amp
    severity: error
  - id: PC-AMP-WARN
    expression: "Features::Topology.maxCpus >= 2"
    appliesWhen: Features::Topology::Amp
    severity: warning
  - id: PC-COMPOUND
    expression: "Features::Topology.maxCpus >= 2"
    appliesWhen: "Features::CortexM33 and Features::Topology::Amp"
    severity: error
  - id: PC-GHOST
    expression: "Features::Topology.ghost > 0"
    severity: error
---
Configs package with cross-feature parameter constraints.
