---
type: Configuration
id: CONF-EVCS-002
name: "Premium tier — 4x90 kW, dual spare"
status: approved
custom_fields:
  mg_variant: true
parameterBindings:
  DesignParameters.converterCount: 4
  DesignParameters.converterPowerKw: 90
  DesignParameters.baseAvailability: 0.990
  DesignParameters.redundancySpare: 2
  DesignParameters.detectionMs: 25
  DesignParameters.contactorTripMs: 20
  DesignParameters.capexPerStall: 120000
  DesignParameters.sessionsLifetime: 90000
  DesignParameters.energyCostPerSession: 2.1
---

**Premium-tier variant.** A four-module 360 kW cabinet with two spare modules.
Higher capex but stronger fast-turnaround and availability effectiveness, faster
fault reaction. A parametric trade-study variant (`mg_variant`) whose
differentiator is its `parameterBindings`.
