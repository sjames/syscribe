---
id: REQ-TRS-PARSE-009
type: Requirement
name: "Tool shall skip files with no type: field"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** skip a file whose frontmatter does not contain a `type:` field, emitting a warning finding. Files with `type:` absent are not model elements and are not included in cross-reference resolution or validation.

**Source:** §11.2 ¶5

**Acceptance criteria:** A `.md` file without `type:` in its frontmatter is skipped with a warning; it does not cause `E004` (which is reserved for required fields other than `type:`).
