---
id: REQ-TRS-VAL-007
type: Requirement
name: Tool shall classify findings as Error or Warning per their rule code
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** assign severity `Error` to all E-code findings and severity `Warning` to all W-code findings, consistently with the classification in §11.12. Severity **shall** be reported alongside each finding.

**Source:** §11.12

**Acceptance criteria:** Every finding in the output is marked as either `Error` or `Warning`; no E-code finding is marked `Warning` and no W-code finding is marked `Error`.
