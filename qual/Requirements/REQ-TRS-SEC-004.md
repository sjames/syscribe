---
id: REQ-TRS-SEC-004
type: Requirement
name: "AssumptionOfUse.appliesTo shall accept CybersecurityGoal and enforce target type"
status: draft
reqDomain: software
verificationMethod: test
---

`AssumptionOfUse.appliesTo` **shall** accept `CybersecurityGoal` as a valid target type, alongside the existing allowed types (`SafetyGoal`, `Argument`, `Requirement`).

When `appliesTo:` refers to an element that resolves but whose type is not one of the allowed set (`SafetyGoal`, `CybersecurityGoal`, `Argument`, `Requirement`), the tool **shall** emit error **`E859`**.

**Acceptance criteria:**

- An AOU element with `appliesTo: CSG-KERNEL-001` (a CybersecurityGoal) validates without E858 or E859.
- Mixing `[SG-001, CSG-001]` in `appliesTo` validates cleanly.
- `appliesTo: SomePart` (resolves to a PartDef) triggers E859.
