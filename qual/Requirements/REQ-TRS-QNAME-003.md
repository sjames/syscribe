---
id: REQ-TRS-QNAME-003
type: Requirement
title: "Element name shall use name: from frontmatter when present"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** use the `name:` field from an element's frontmatter as that element's name segment in its qualified name, overriding the filename stem.

**Source:** §11.3 ¶3

**Acceptance criteria:** A file `model/Engine.md` with `name: InternalCombustionEngine` in its frontmatter has qualified name `InternalCombustionEngine`, not `Engine`.
