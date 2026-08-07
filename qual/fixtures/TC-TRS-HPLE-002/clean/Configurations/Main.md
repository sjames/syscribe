---
type: Configuration
id: CONF-MAIN-CLEAN-002
name: "Main — binds a peer parameter reachable through subConfigurations"
status: approved
featureModel: Features
features:
  Features: true
subConfigurations: CONF-PEER-CARGO-001
parameterBindings:
  Features::Cargo.capacityKg: 2.0
---
Binds the peer's open parameter using its ordinary, already-mounted qname — must resolve cleanly.
