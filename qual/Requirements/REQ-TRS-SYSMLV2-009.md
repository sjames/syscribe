---
id: REQ-TRS-SYSMLV2-009
type: Requirement
name: SysML v2 doc /* ... */ comments shall lift into the synthesized element's doc body
status: draft
reqDomain: software
verificationMethod: test
---

A `part def`/`part`/`interface def`/`port def`/`port`/`connection def`/`attribute def`/
`attribute`/`item def` **shall** be able to declare one or more `doc /* ... */` members. The tool
**shall** lift their text, concatenated in source order (joined by a blank line, each block's own
text trimmed of delimiter-adjacent whitespace), into the synthesized element's `doc` field — the
same field a hand-authored element's body populates — so `W600`/`W601`-style empty-doc-body
checks apply unchanged.

A `part def`/`part`/etc. with **no** `doc` member **shall** be ingested exactly as it is today —
`doc: ""`, `W600` still fires, no regression.

**Source:** `REQ-TRS-SYSMLV2-009` (product model).

**Acceptance criteria:** a `part def` with `doc /* Explanation. */` gets `doc: "Explanation."` on
the synthesized element and clears `W600`; a `part def` with two `doc` blocks gets both texts
concatenated in source order; a `part def`/`part`/etc. with no `doc` member is ingested exactly as
it is today (`doc: ""`, `W600` still fires).
