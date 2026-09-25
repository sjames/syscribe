---
id: REQ-TRS-QNAME-003
type: Requirement
name: "Element qualified-name segment shall be the filename stem; name: is a label only"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** use a file's stem (its filename without `.md`) as that element's name segment in its qualified name. The element's `name:` field **shall** be treated as a display label only and **shall not** change its qualified name.

**Source:** §11.3 step 3; §4.2

**Acceptance criteria:** A file `model/Engine.md` with `name: InternalCombustionEngine` in its frontmatter has qualified name `Engine`, not `InternalCombustionEngine`.
