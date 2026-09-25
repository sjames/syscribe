---
type: Configuration
id: CONF-TOP-OK-001
name: "Top variant closing the remaining open parameter two tiers down"
status: approved
featureModel: Features
features:
  Features: true
subConfigurations: Supply::PackConfs::CONF-PACK-001
parameterBindings:
  Features::Cell.siteCode: US
---
Consolidates the pack tier through the `Supply::PackConfs` mount and closes `siteCode`; with the
pack tier's mount-path binding of `capacityAh`, the subtree is fully closed.
