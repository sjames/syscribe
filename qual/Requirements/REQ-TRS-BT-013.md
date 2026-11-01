---
id: REQ-TRS-BT-013
type: Requirement
name: "A derived Configuration's stage is the same as or later than its base's"
status: draft
reqDomain: software
verificationMethod: test
---

When a Configuration and its `derivedFrom:` base (§9.8) are both staged, the derived Configuration's `bindingStage:` shall be the same as or later than the base's; an earlier one shall be an error (`E753`).

**Acceptance criteria:**
- A `compile` Configuration derived from a `load` Configuration is an error; the reverse is clean.
