---
type: FeatureDef
id: FEAT-CELL
name: Cell
groupKind: mandatory
parameters:
  - name: nominalVoltageV
    type: ScalarValues::Real
    default: 3.2
  - name: cycleLifeRating
    type: ScalarValues::Real
    range: "300..3000"
    isRequired: true
  - name: manufacturingSiteCode
    type: ScalarValues::String
    enumValues: [CN, US, DE]
    isRequired: true
---

The cell chemistry itself. Three parameters, deliberately spanning every case this worked example
demonstrates:

- **`nominalVoltageV`** carries a sensible `default:` (this chemistry's fixed nominal cell voltage)
  and is never touched by any consolidating tier — self-sufficient by construction
  (`REQ-TRS-HPLE-004`'s scope: "a parameter a tier author deliberately supplied a `default:` for...
  is not something anyone up the chain needs to decide about").
- **`cycleLifeRating`** is `isRequired: true` with no `default:` — genuinely open. The battery-pack
  tier (one level up) closes it directly: the acceptable cycle life is a pack-design decision, not
  something the cell chemistry itself can know in advance.
- **`manufacturingSiteCode`** is also open, but the battery-pack tier deliberately leaves it
  unset — sourcing site is a vehicle-program decision, not a pack-design one. The vehicle tier (two
  levels up) closes it directly, reaching straight through the pack tier via `manufacturingSiteCode`'s
  ordinary, already-mounted qname — no new addressing syntax (`REQ-TRS-HPLE-002`).
