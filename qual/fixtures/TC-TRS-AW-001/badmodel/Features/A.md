---
type: FeatureDef
id: FEAT-AA
name: A
mandatory: true
excludes:
  - Features::B
---
Mandatory feature A that excludes mandatory feature B (contradiction -> void).
