---
id: REQ-TRS-SYSMLV2-012
type: Requirement
name: A named connection usage's own trailing doc /* ... */ body shall lift into the synthesized Connection element's doc field
status: draft
reqDomain: software
verificationMethod: test
---

A named `connection name : Type connect a to b { doc /* ... */ }` usage's own trailing body
**shall** have its `doc /* ... */` member(s) lifted into the synthesized `Connection` element's
`doc` field, reusing the same lift `REQ-TRS-SYSMLV2-009` already gives a `connection def { }`. A
connection usage with no trailing body **shall** be ingested exactly as it is today — no
regression.

**Source:** `REQ-TRS-SYSMLV2-012` (product model).

**Acceptance criteria:** `connection c : SomeDef connect a to b { doc /* Explanation. */ }` lifts
`doc: "Explanation."` onto the synthesized `Connection` element, retrievable via `show`; a
connection usage with no trailing body has an empty `doc`, unaffected.
