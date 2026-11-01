---
type: FeatureDef
id: FEAT-FX-B86
name: Comp
groupKind: optional
parameters:
  - name: kv
    type: ScalarValues::Real
    range: "0..100"
    bindingTime: compile
    bindTo: "Features::Sys.kv"
---
A fixture feature.
