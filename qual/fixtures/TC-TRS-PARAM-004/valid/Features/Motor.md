---
type: FeatureDef
id: FEAT-FX-091
name: Motor
groupKind: optional
parameters:
  - name: kv
    type: ScalarValues::Real
    range: "900..1200"
    isRequired: true
    bindingTime: compile
  - name: spin
    type: ScalarValues::Real
    range: "0..100"
    bindingTime: load
  - name: adapt
    type: ScalarValues::Real
    range: "0..10"
    bindingTime: runtime
  - name: cool
    type: ScalarValues::Boolean
    derivedFrom: "kv > 1100"
    bindingTime: runtime
---
A feature with well-ordered binding times across all parameter kinds.
