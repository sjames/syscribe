---
id: REQ-TRS-OUT-004
type: Requirement
name: Tool shall exit with a non-zero code when Error findings are present
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** exit with a non-zero exit code (≥ 1) when the model contains one or more `Error`-severity findings. This enables use of the tool as a CI gate.

**Source:** §11.7

**Acceptance criteria:** A model with at least one E-code finding causes the tool to exit with a non-zero code verifiable via `$?` in the shell.
