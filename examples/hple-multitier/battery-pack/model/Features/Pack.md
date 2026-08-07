---
type: FeatureDef
id: FEAT-PACK
name: Pack
groupKind: mandatory
parameters:
  - name: packCapacityKwh
    type: ScalarValues::Real
    range: "20..150"
    isRequired: true
---

The assembled pack. `packCapacityKwh` is open — total pack energy capacity is a vehicle-program
decision (it depends on the target vehicle's required range), not something a reusable pack design
can fix in advance. This tier leaves it exactly as open as `Cell.manufacturingSiteCode` — declared
`isRequired: true` with no `default:`, no upward-pointing field (`REQ-TRS-HPLE-005`).
