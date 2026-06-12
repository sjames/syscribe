---
type: Configuration
id: CONF-PC-SMP-001
name: "SMP single core — AMP constraints do not apply"
status: approved
featureModel: Features
features:
  Features::Topology: true
  Features::Topology::Amp: false
  Features::Topology::Smp: true
  Features::CortexM33: true
parameterBindings:
  Features::Topology.maxCpus: 1
---
SMP (not AMP) with maxCpus = 1 — the AMP-gated constraints must not be evaluated here.
