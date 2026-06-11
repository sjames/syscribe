---
id: REQ-TRS-PARSE-004
type: Requirement
title: Tool shall honor .sysmlignore exclusion patterns
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** read a `.sysmlignore` file at the model root, if present, and exclude any file or directory matching the glob patterns listed therein (one pattern per line, gitignore syntax).

**Source:** §11.1 ¶4

**Acceptance criteria:** A `.sysmlignore` entry suppresses processing of the matching file(s); absence of `.sysmlignore` does not cause an error.
