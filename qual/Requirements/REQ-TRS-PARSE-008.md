---
id: REQ-TRS-PARSE-008
type: Requirement
name: Tool shall parse frontmatter content as YAML 1.2
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** parse the content between the frontmatter delimiters as YAML 1.2. A frontmatter block that is not valid YAML 1.2 **shall** cause error `E002` to be emitted for that file.

**Source:** §11.2 ¶2; §11.12 `E002`

**Acceptance criteria:** A file with invalid YAML (e.g. unbalanced braces, duplicate keys) produces exactly one `E002` finding.
