---
id: REQ-TRS-SEC-005
type: Requirement
name: "ConfirmationMeasure.confirms shall accept CybersecurityGoal as a valid target"
status: draft
reqDomain: software
verificationMethod: test
---

`ConfirmationMeasure.confirms` **shall** accept `CybersecurityGoal` as a valid target type, in addition to the existing accepted types (`SafetyGoal`, `HazardousEvent`, `Requirement`).

When `confirms:` refers to an element that resolves but is not one of the allowed types, the tool **shall** emit error **`E860`** ("ConfirmationMeasure.confirms '{}' is not a valid confirmation target type").

**Acceptance criteria:**

- A CM with `confirms: CSG-KERNEL-001` does not trigger E851 or E860.
- A CM with `confirms: [SG-001, CSG-001]` validates cleanly.
- A CM with `confirms: SomePart` (PartDef) triggers E860.
