---
type: FeatureDef
id: FEAT-BATTERY-CELL
name: Cell
groupKind: mandatory
parameters:
  - name: voltage
    type: ScalarValues::Real
    bindTo: "Features::TopSecret.forbidden"
    range: "0..10"
---
`bindTo:` names a dotted path that exists nowhere in this model, but coincidentally matches a
real parameter in a separate, higher-tier model — the isolation claim under test.
