---
type: FeatureDef
id: FEAT-FX-003
name: Battery
groupKind: optional
parameters:
  - name: capacityKwh
    type: ScalarValues::Real
    range: "20.0..100.0"
    isRequired: true
---

High-voltage traction battery. The usable energy capacity is a configurable
parameter bound per product.
