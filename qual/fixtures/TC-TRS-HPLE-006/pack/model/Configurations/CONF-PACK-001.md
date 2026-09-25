---
type: Configuration
id: CONF-PACK-001
name: "Pack variant consolidating the cell tier through mount paths"
status: approved
featureModel: Features
features:
  Features: true
subConfigurations: Vendor::CellConfs::CONF-CELL-001
parameterBindings:
  Vendor::CellFeatures::Cell.capacityAh: 50
---
Names the consolidated cell Configuration and closes `capacityAh` through the `Vendor::CellConfs`
/ `Vendor::CellFeatures` mounts; `siteCode` is deliberately left open (W513).
