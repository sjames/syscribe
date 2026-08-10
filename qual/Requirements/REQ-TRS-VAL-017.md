---
id: REQ-TRS-VAL-017
type: Requirement
name: W600 shall be suppressed on a Part usage whose typedBy target already carries non-empty documentation
status: draft
reqDomain: software
verificationMethod: test
---

`W600` **shall not** fire on a `type: Part` element with an empty `doc` body when its `typedBy:`
resolves to another element with a non-empty `doc` body. `W600` **shall** continue to fire for: a
`PartDef` itself with an empty `doc`; a `Part` usage whose `typedBy:` target also has an empty
`doc`; and a `Part` usage whose `typedBy:` doesn't resolve.

**Source:** `REQ-TRS-VAL-017` (product model).

**Acceptance criteria:** a `Part` with an empty `doc` and `typedBy:` pointing at a documented
`PartDef` raises no `W600`; the documented `PartDef` itself raises no `W600` (it has its own doc);
an undocumented `PartDef` raises `W600`; a `Part` typed by that undocumented `PartDef` also raises
`W600`; a `Part` with an unresolvable `typedBy:` raises `W600`.
