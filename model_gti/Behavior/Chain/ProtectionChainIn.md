---
type: PartDef
name: ProtectionChainIn
supertype: Behavior::Chain::ProtectionChain
features:
  - {name: monitor, typedBy: Behavior::Parts::GridMonitorPartIn}
  - {name: supervisor, typedBy: Behavior::Parts::SupervisorPartIn}
---

India variant of the protection chain: same wiring, the India monitor limits and the 60 s reconnect delay.
