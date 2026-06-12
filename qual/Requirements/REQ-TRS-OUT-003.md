---
id: REQ-TRS-OUT-003
type: Requirement
name: Report shall include a summary count of errors and warnings
status: draft
reqDomain: software
verificationMethod: test
---

The tool's output report **shall** include a summary section stating the total number of `Error`-severity findings and the total number of `Warning`-severity findings found in the model.

**Source:** §11.7

**Acceptance criteria:** The summary counts match the number of E-code and W-code findings listed in the report body.
