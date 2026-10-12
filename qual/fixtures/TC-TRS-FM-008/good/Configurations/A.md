---
type: Configuration
id: CONF-Q8-AAA-001
name: A
status: draft
featureModel: Features
features:
  Features::Protection: true
  FEAT-Q8-AFCI: true
  Features::Protection::Rsd: true
  Features::Mppt: true
  Features::Mppt::GlobalScan: true
parameterBindings:
  Features::Mppt::GlobalScan.scanPeriodS: 300
---
