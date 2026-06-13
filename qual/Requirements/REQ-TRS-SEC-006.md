---
id: REQ-TRS-SEC-006
type: Requirement
name: "Requirement shall support derivedFromCybersecurityGoal field for ISO/SAE 21434 traceability"
status: draft
reqDomain: software
verificationMethod: test
---

The `Requirement` element **shall** support a field **`derivedFromCybersecurityGoal`** (string, scalar, YAML key) that references the `CybersecurityGoal` element from which the requirement is derived, per ISO/SAE 21434 §14.

The field **shall** be backward-compatible with the existing `derivedFromSecurityGoal` YAML key as a serde alias.

The validator **shall** emit error **`E831`** (repurposed: previously `derivedFromSecurityGoal`, now `derivedFromCybersecurityGoal`) when the referenced element cannot be resolved, or resolves to a non-`CybersecurityGoal` element.

**Acceptance criteria:**

- `derivedFromCybersecurityGoal: CSG-001` resolves cleanly when CSG-001 exists.
- `derivedFromSecurityGoal: CSG-001` (old name) also resolves without error (backward compat).
- A dangling `derivedFromCybersecurityGoal: CSG-NONEXIST` triggers E831.
- A `derivedFromCybersecurityGoal:` pointing to a non-CSG element triggers E831.
