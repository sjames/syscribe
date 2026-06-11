---
id: REQ-TRS-CLI-002
type: Requirement
title: Tool shall emit an error to stderr and exit non-zero if the model directory is invalid
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** emit a human-readable error message to `stderr` and exit with a non-zero exit code if the path supplied via `-m` / `--model` does not exist, is not a directory, or cannot be read.

**Source:** §11.1

**Acceptance criteria:** `syscribe -m /nonexistent` exits non-zero with an error message on stderr and produces no Markdown report on stdout.
