---
id: REQ-TRS-OUT-005
type: Requirement
title: Tool shall exit with code 0 when no Error findings are present
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** exit with exit code `0` when the model contains no `Error`-severity findings. `Warning`-only findings **shall not** cause a non-zero exit.

**Source:** §11.7

**Acceptance criteria:** A valid model with only W-code findings (or no findings at all) causes the tool to exit with code `0`.
