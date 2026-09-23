---
id: REQ-TRS-LINKTYPE-003
type: Requirement
name: "A link type's declared sourceTypes and targetTypes shall be enforced"
status: draft
reqDomain: software
verificationMethod: test
---

When a link type declares `sourceTypes`, a `links:` entry of that type on an element whose
`type:` is not listed **shall** raise error `E633`. When it declares `targetTypes`, each
resolved target whose `type:` is not listed **shall** raise error `E634` naming the target.
Omitting either list permits any type on that end.

**Acceptance criteria:** a permitted source/target validates clean; a forbidden source raises
`E633`; a forbidden target raises `E634`.

**Source:** `REQ-TRS-LINKTYPE-003` (product model), `ADR-SYS-LINKTYPE-001`.
