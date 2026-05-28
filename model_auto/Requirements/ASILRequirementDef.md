---
type: RequirementDef
name: ASIL Requirement
features:
  - name: asilLevel
    type: ScalarValues::String
  - name: verificationMethod
    type: ScalarValues::String
  - name: safetyMechanism
    type: ScalarValues::String
---

Requirement definition template for ISO 26262 safety requirements. All
concrete ASIL-rated requirements in this model are instances of this type,
inheriting the `asilLevel`, `verificationMethod`, and `safetyMechanism`
attributes.

Used by requirements in `Requirements/Safety/` that carry an `asilLevel:`
field.
