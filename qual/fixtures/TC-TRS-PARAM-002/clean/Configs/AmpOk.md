---
type: Configuration
id: CONF-PC-AMPOK-001
name: "AMP with four cores (satisfies >= 2)"
status: approved
featureModel: Features
features:
  Features::Topology: true
  Features::Topology::Amp: true
  Features::Topology::Smp: false
  Features::CortexM33: true
parameterBindings:
  Features::Topology.maxCpus: 4
---
AMP + CortexM33 with maxCpus = 4 — satisfies every constraint.
