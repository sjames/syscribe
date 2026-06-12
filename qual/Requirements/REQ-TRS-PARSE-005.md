---
id: REQ-TRS-PARSE-005
type: Requirement
name: Tool shall treat _index.md as the package declaration for its directory
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** treat a file named `_index.md` as the package declaration for its containing directory. The `_index.md` file **shall not** contribute a name segment to the qualified names of sibling elements.

**Source:** §11.1 ¶5, §11.3 ¶4

**Acceptance criteria:** `_index.md` metadata (e.g. `name:`) is applied to the package, not treated as a child element.
