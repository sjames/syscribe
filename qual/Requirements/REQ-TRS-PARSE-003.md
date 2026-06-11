---
id: REQ-TRS-PARSE-003
type: Requirement
title: Tool shall ignore standard build and tool directories
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** exclude the following directories and their contents from file discovery: `.git/`, `.github/`, `node_modules/`, `target/`, `dist/`, and any directory or file whose name begins with `.` (hidden files).

**Source:** §11.1 ¶4

**Acceptance criteria:** Files placed under `target/` or `.git/` within the model root are not processed.
