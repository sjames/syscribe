---
type: FeatureDef
id: FEAT-FX-089
name: Motor
groupKind: optional
parameters:
  - name: kv
    type: ScalarValues::Real
    range: "900..1200"
    bindingTime: runtime
  - name: cool
    type: ScalarValues::Boolean
    derivedFrom: "kv > 1100"
    bindingTime: compile
---
A compile-time parameter derived from a runtime sibling — impossible ordering (E229).
