---
type: FaultTreeGate
id: FTG-TST-001
title: Gate With Unresolvable Input
gateType: AND
inputs:
  - NonExistentEvent
---

The `inputs` entry `NonExistentEvent` does not resolve to any element — triggers E906.
