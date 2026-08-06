---
type: Configuration
id: CONF-HEX-DRONE-001
name: "Hex-rotor product build"
status: approved
featureModel: Features
features:
  Features::RotorConfig: true
  Features::RotorConfig::Quad: false
  Features::RotorConfig::Hex: true
---

Redundant hex-rotor product variant. Projecting this configuration includes the
SysML v2 `hexConfig` variant part and excludes `quadConfig` — the mirror image
of `CONF-QUAD-DRONE-001`.
