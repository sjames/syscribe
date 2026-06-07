---
type: Configuration
id: CONF-PC-VIOL-001
title: "AMP bound to a single core (violates >= 2)"
status: approved
featureModel: Features
features:
  Features::Topology: true
  Features::Topology::Amp: true
  Features::Topology::Smp: false
  Features::CortexM33: true
parameterBindings:
  Features::Topology.maxCpus: 1
---
AMP + CortexM33 but maxCpus = 1 — violates PC-AMP-MIN, PC-AMP-WARN, PC-COMPOUND.
