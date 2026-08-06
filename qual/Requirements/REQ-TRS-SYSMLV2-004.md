---
id: REQ-TRS-SYSMLV2-004
type: Requirement
name: A native TestCase's verifies field shall be able to target a SysMLv2-originated element
status: draft
reqDomain: software
verificationMethod: test
---

A native Syscribe `TestCase`'s existing `verifies:` field **shall** also resolve against the
qname index of any ingested SysMLv2 subtree, so a `TestCase` can verify a SysMLv2-authored
element (a `Part`, a `Requirement` usage, etc.) the same way it verifies a native `Requirement`
today — with no change to `TestCase`'s schema or its existing validation rules beyond widening
what `verifies:` is allowed to resolve to. This widening **shall** be scoped to elements that
actually came from SysMLv2 ingestion: a hand-authored native element of the same kind (e.g. a
plain `PartDef`) **shall** continue to be rejected exactly as before.

**Source:** `REQ-TRS-SYSMLV2-004` (product model).

**Acceptance criteria:** a `TestCase.verifies:` naming a SysMLv2-mapped element's qualified name
resolves with no dangling-reference or wrong-type finding, and that qualified name appears in the
tool's computed reverse coverage index; the same mechanism continues to work, unchanged, for a
`TestCase` verifying a native `Requirement`; a `TestCase.verifies:` naming a hand-authored native
element that is not a `Requirement` still produces the existing wrong-type finding.
