---
id: REQ-TRS-OUT-001
type: Requirement
name: Tool shall output a validation report in Markdown format to stdout
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** write its validation report to standard output (`stdout`) in Markdown format when invoked with a model directory. The report **shall** be machine-parseable and human-readable.

**Source:** §11.7

**Acceptance criteria:** Redirecting stdout to a file produces a valid Markdown document. Stderr is not mixed into the report.
