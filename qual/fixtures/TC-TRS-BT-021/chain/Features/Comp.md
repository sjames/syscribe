---
type: FeatureDef
id: FEAT-FX-PV2
name: Comp
groupKind: optional
parameters:
  - name: kv
    type: ScalarValues::Real
    range: "900..1200"
    bindTo: "Features::Motor.kv"
---
A component whose parameter is propagated from the system parameter.
