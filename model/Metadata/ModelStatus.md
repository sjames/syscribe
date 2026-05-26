---
type: MetadataDef
name: ModelStatus
annotates:
  - Element
isSemantic: false
features:
  - name: status
    typedBy: ScalarValues::String
    value: "draft"
    valueKind: default-initial
  - name: reviewer
    typedBy: ScalarValues::String
---

Model review status annotation tracking element maturity through draft, review, approved, and deprecated states.
