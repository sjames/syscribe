---
type: Configuration
id: CONF-QUAD-DRONE-001
name: "Quad-rotor product build"
status: approved
featureModel: Features
features:
  Features::RotorConfig: true
  Features::RotorConfig::Quad: true
  Features::RotorConfig::Hex: false
---

Lightweight quad-rotor product variant. Projecting this configuration
(`syscribe validate --config CONF-QUAD-DRONE-001` / `syscribe diagram --config
CONF-QUAD-DRONE-001`) includes the SysML v2 `quadConfig` variant part and
excludes `hexConfig`.
