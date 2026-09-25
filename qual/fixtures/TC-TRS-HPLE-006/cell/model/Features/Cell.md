---
type: FeatureDef
id: FEAT-CELL
name: Cell
groupKind: optional
parameters:
  - name: capacityAh
    type: ScalarValues::Real
    range: "10..100"
    isRequired: true
  - name: siteCode
    type: ScalarValues::String
    isRequired: true
---
Two open parameters: `capacityAh` is closed by the pack tier (through its mount path),
`siteCode` by the top tier.
