---
type: FeatureDef
id: FEAT-FX-B88
name: Comp
groupKind: optional
parameters:
  - name: kv
    type: ScalarValues::Real
    range: "0..100"
    bindingTime: load
    bindTo: "Features::Sys.kv"
---
A fixture feature.
