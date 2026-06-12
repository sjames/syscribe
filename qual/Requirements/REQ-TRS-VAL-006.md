---
id: REQ-TRS-VAL-006
type: Requirement
name: Parse-time errors shall be attributed to the file that triggered them
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** attribute each parse-time error (E001–E015, E300–E304) to the specific file from which it originated. The file path **shall** be included in the finding.

**Source:** §11.7

**Acceptance criteria:** A model with malformed frontmatter in `model/A/B.md` produces a finding that references `model/A/B.md` or its derived qualified name, not a generic location.
