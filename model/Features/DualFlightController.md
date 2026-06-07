---
type: FeatureDef
name: DualFlightController
groupKind: optional
requires:
  - Features::Propulsion::Hex
---

Redundant dual flight-controller configuration with automatic failover. Requires
the **Hex** platform so that flight can be sustained through a single failure.
