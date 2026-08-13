---
id: REQ-TRS-SYSMLV2-017
type: Requirement
name: W007 usage tracking and graph.rs's TypedBy edge resolve a package-relative typedBy/supertype reference across SysMLv2 packages
status: draft
reqDomain: software
verificationMethod: test
---

`W007`'s "defined but never used as a supertype or type" usage tracking, and `graph.rs`'s
`TypedBy` edge, **shall** resolve a package-relative `typedBy:`/`supertype:` reference (as a
`.sysml` author actually writes it, e.g. `Services::Documented` from inside a different package
than `Documented`'s own) by searching outward through the referencing element's enclosing-package
scope chain, not only by an exact match against the target's full model-root qname. A `*Def`
referenced only this way **shall** count as used (no `W007`) and **shall** be a real,
`connectivity`-traversable `TypedBy` graph edge. A genuinely unused `*Def` **shall** still raise
`W007`.

**Source:** `REQ-TRS-SYSMLV2-017` (product model), `ADR-SYS-SYSMLV2-001` addendum.
