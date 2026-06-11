---
type: PartDef
name: BrakeFunction
supertype: Physical::BrakeECU
custom_fields:
  mg_layer: logical
---

A logical part whose supertype is a physical part → MG042 (cross-layer
coupling). The supertype edge crosses the logical/physical boundary directly
instead of going through an Allocation.
