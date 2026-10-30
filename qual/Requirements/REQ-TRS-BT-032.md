---
id: REQ-TRS-BT-032
type: Requirement
name: "A dependent that relies on a retained element being absent is reported"
status: draft
reqDomain: software
verificationMethod: test
---

A requirement, `satisfies:` link or test that relies on the absence of a variant-retained element shall produce a finding (`W752`): an element that references a retained one, yet applies only where it does not (its condition and the retained element's can never hold together).

**Acceptance criteria:**
- A test that verifies the absence of a retained element is reported; one that does not is silent.
