---
id: REQ-TRS-PARSE-001
type: Requirement
name: Tool shall accept a model root directory
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** accept a single model root directory path as input and use that directory as the root namespace for all subsequent file discovery and qualified name derivation.

**Source:** §11.1 ¶1

**Acceptance criteria:** Invoking the tool with a valid directory path produces a validation report; invoking it with no path or a non-directory path produces an error.
