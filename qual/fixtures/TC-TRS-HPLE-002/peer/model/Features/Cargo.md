---
type: FeatureDef
id: FEAT-PEER-CARGO
name: Cargo
groupKind: optional
parameters:
  - name: capacityKg
    type: ScalarValues::Real
    range: "0.5..5.0"
    isRequired: true
---
An open parameter, reachable only through subConfigurations from a consolidating tier.
