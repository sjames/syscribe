---
type: MetadataDef
name: DALAnnotation
annotates:
  - PartDef
  - ActionDef
isSemantic: true
features:
  - name: level
    typedBy: ScalarValues::Integer
    value: "4"
    valueKind: default-initial
  - name: rationale
    typedBy: ScalarValues::String
    value: ""
    valueKind: default-initial
---

DO-178C Design Assurance Level annotation. Level 1 = most critical (catastrophic failure condition), Level 5 = no safety effect.
