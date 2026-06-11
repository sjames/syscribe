---
id: REQ-TRS-PARSE-007
type: Requirement
title: Tool shall require frontmatter to begin on the first line
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognize YAML frontmatter delimited by a `---` line that is the first line of the file (no preceding whitespace or blank lines). The closing `---` delimiter ends the frontmatter block. Content after the closing `---` is the Markdown documentation body.

**Source:** §11.2 ¶1

**Acceptance criteria:** A file whose first line is `---` is parsed correctly; a file with a blank first line is treated as having no frontmatter (warning emitted).
