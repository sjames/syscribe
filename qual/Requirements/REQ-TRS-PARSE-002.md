---
id: REQ-TRS-PARSE-002
type: Requirement
title: Tool shall recursively process all .md files under the model root
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recursively walk all subdirectories of the model root and treat every file with a `.md` extension as a candidate model element file.

**Source:** §11.1 ¶2–3

**Acceptance criteria:** A model with elements in deeply nested subdirectories is discovered and processed correctly.
