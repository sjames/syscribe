---
type: Configuration
id: CONF-MG-EXACT-001
name: ExactConf
status: approved
parameterBindings:
  SubsysA.speed: 10
  SubsysB.speed: 20
  speed: 99
custom_fields:
  mg_variant: true
---
Exact `speed` key wins -> 99.
