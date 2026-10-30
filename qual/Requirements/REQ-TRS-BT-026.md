---
id: REQ-TRS-BT-026
type: Requirement
name: "A parameter may declare a fallback value that is validated and exported"
status: draft
reqDomain: software
verificationMethod: test
---

A parameter shall accept an optional `fallback:` — the safe value when a late binding is missing or invalid — validated against its `range:` and `enumValues:` as an error (`E759`, reported once each by `validate` and `feature-check`), and emitted by `params export` as a comment.

**Acceptance criteria:**
- A `fallback:` outside `range:` is an error.
- A valid `fallback:` appears in the export.
