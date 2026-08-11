---
id: REQ-TRS-SYSMLV2-016
type: Requirement
name: REQ-TRS-VAL-017's W600 typedBy documentation fallback resolves a package-relative typedBy reference across SysMLv2 packages
status: draft
reqDomain: software
verificationMethod: test
---

`W600`'s `typedBy:` documentation-fallback suppression **shall** resolve a package-relative
`typedBy:` reference (as a `.sysml` author actually writes it, e.g. `Services::Documented` from
inside a different package than `Documented`'s own) by searching outward through the referencing
element's enclosing-package scope chain, not only by an exact match against the target's full
model-root qname. A `Part` usage whose (package-relative) `typedBy:` resolves this way to a
documented target **shall** suppress `W600`, exactly like the already-correct same-package case.

**Source:** `REQ-TRS-SYSMLV2-016` (product model), `ADR-SYS-SYSMLV2-001` addendum.
