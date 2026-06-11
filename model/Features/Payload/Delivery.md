---
type: FeatureDef
id: FEAT-DELIVERY
name: Delivery
groupKind: optional
requires:
  - Features::Propulsion::Hex
parameters:
  - name: payloadCapacityKg
    type: ScalarValues::Real
    range: "0.5..5.0"
    isRequired: true
---

Cargo delivery payload with a release mechanism. Requires the **Hex** platform
for the necessary lift capacity. The maximum cargo mass is a configurable
parameter bound per product.
