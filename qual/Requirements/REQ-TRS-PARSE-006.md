---
id: REQ-TRS-PARSE-006
type: Requirement
title: Tool shall continue processing after a file with unparseable frontmatter
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** emit a warning finding and skip a file whose YAML frontmatter is absent or cannot be parsed. This condition **shall not** cause a fatal error or halt processing of remaining files.

**Source:** §11.1 ¶6, §11.2 ¶4

**Acceptance criteria:** A model containing one malformed file produces warnings for that file and correct results for all other files.
