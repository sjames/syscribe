---
type: Configuration
id: CONF-EVCS-001
name: "Entry tier — 2x90 kW, no spare"
status: approved
custom_fields:
  mg_variant: true
parameterBindings:
  DesignParameters.converterCount: 2
  DesignParameters.converterPowerKw: 90
  DesignParameters.baseAvailability: 0.985
  DesignParameters.redundancySpare: 1
  DesignParameters.detectionMs: 40
  DesignParameters.contactorTripMs: 35
  DesignParameters.capexPerStall: 70000
  DesignParameters.sessionsLifetime: 60000
  DesignParameters.energyCostPerSession: 1.8
---

**Entry-tier variant.** A two-module 180 kW cabinet with one spare module for
redundancy. Lower capex per stall, modest availability uplift. A parametric
trade-study variant (`mg_variant`) whose differentiator is its
`parameterBindings`.
