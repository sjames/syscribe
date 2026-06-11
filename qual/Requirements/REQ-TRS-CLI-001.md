---
id: REQ-TRS-CLI-001
type: Requirement
title: Tool shall accept the model directory via -m / --model argument
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** accept the model root directory path via the `-m` / `--model` command-line argument. The argument **shall** be mandatory for validation mode.

**Source:** CLAUDE.md §Common Commands

**Acceptance criteria:** `syscribe -m model/` and `syscribe --model model/` both invoke validation against the specified directory.
