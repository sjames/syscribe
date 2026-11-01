---
id: REQ-TRS-BT-038
type: Requirement
name: "A model that uses none of the binding-time extensions validates and reports exactly as before"
status: draft
reqDomain: software
verificationMethod: test
---

A model that uses none of the new fields and declares no `[bindingTimes]` shall validate, project and report exactly as it does without this feature, and no new finding shall appear on `model/` or the existing examples after any phase of delivery.

**Acceptance criteria:**
- `validate` on `model/` and every `examples/` model yields no new finding: the findings of the pre-feature build and the current build are identical on every existing model (verified by diffing both builds' output; `TC-TRS-BT-038` asserts none raises a binding-time code).
- The existing variability test suites pass unchanged.
