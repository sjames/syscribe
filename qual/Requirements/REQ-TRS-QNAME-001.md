---
id: REQ-TRS-QNAME-001
type: Requirement
name: Tool shall derive qualified names from directory path and filename stem
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** derive the qualified name of an element as the `::` -separated sequence of path segments from the model root to the file, using the directory names as namespace segments and the filename stem (filename without `.md`) as the element name.

**Source:** §11.3

**Acceptance criteria:** A file at `model/A/B/C.md` has qualified name `A::B::C`. A file at model root `model/X.md` has qualified name `X`.
