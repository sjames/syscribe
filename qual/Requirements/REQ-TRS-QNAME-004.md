---
id: REQ-TRS-QNAME-004
type: Requirement
title: _index.md shall not contribute a name segment to sibling qualified names
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall not** include `_index` as a name segment in the qualified name of any element. `_index.md` represents the package for its directory; it does not add a namespace level.

**Source:** §11.3 ¶4

**Acceptance criteria:** `model/Pkg/_index.md` with `type: Package` has qualified name `Pkg`; a sibling file `model/Pkg/Foo.md` has qualified name `Pkg::Foo`, not `Pkg::_index::Foo`.
