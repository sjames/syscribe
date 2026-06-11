---
type: FeatureDef
id: FEAT-FX-090
name: Motor
groupKind: optional
parameters:
  - name: kv
    type: ScalarValues::Real
    range: "900..1200"
    bindingTime: compile
  - name: cool
    type: ScalarValues::Boolean
    derivedFrom: "kv > 1100"
    bindingTime: runtime
---
A runtime parameter derived from a compile-time sibling — valid ordering (no E229).
