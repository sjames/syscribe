---
type: Configuration
id: CONF-TOP-BAD-001
name: "Top variant naming a mount path that resolves to nothing in the peer"
status: approved
featureModel: Features
features:
  Features: true
subConfigurations: Supply::PackConfs::CONF-NOPE-001
---
`Supply::PackConfs::CONF-NOPE-001` translates to the pack tier's `Configurations::CONF-NOPE-001`,
which does not exist: E516.
