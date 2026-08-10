---
id: REQ-TRS-SYSMLV2-011
type: Requirement
name: n2's subpart axis shall include a scope's synthesized SysMLv2 children via containment, not only features:-declared subparts
status: draft
reqDomain: software
verificationMethod: test
---

`n2 <qname>`'s subpart-axis selection **shall** include, alongside its existing
`features:`-declared subparts, every `PartDef`/`Part` whose qualified name is a direct child of
the scope element — so a `sysmlSubmodel: true` subtree's SysMLv2-synthesized children populate
`n2`'s axis, and a lifted connection edge between two such children populates the corresponding
off-diagonal cell. Unscoped `n2` and a `features:`-only hand-authored model **shall** be
unaffected — no regression.

**Source:** `REQ-TRS-SYSMLV2-011` (product model).

**Acceptance criteria:** `n2 <sysmlv2-subtree-root>` lists every direct-child `Part` of that root
on the diagonal; a lifted connection between two such parts populates the corresponding
off-diagonal cell; the existing `features:`-only native `n2` behavior (`REQ-TRS-OUT-016`) is
unaffected.
