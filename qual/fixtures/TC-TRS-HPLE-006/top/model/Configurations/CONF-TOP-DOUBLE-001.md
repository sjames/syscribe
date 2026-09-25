---
type: Configuration
id: CONF-TOP-DOUBLE-001
name: "Top variant re-binding a parameter the pack tier already closed"
status: approved
featureModel: Features
features:
  Features: true
subConfigurations: Supply::PackConfs::CONF-PACK-001
parameterBindings:
  Features::Cell.siteCode: US
  Features::Cell.capacityAh: 60
---
`capacityAh` is already closed by the pack tier (through that tier's own `Vendor::CellFeatures`
mount): re-binding it here is E523.
