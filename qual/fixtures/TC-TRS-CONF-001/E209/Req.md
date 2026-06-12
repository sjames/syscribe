---
id: REQ-TST-001
type: Requirement
name: Requirement with appliesWhen pointing to a Part
status: draft
reqDomain: software
verificationMethod: test
appliesWhen: MyPart
---

This requirement **shall** be conditionally applicable. The `appliesWhen:` references MyPart which is a Part, not a FeatureDef — should produce E209.
