---
type: TestCase
id: TC-TRS-MCP-040
name: "lint_docs returns unresolvable-reference findings for prose and SVG"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_diagrams.rs
verifies:
  - REQ-TRS-MCP-039
tags:
  - mcp
---

```gherkin
Feature: Documentation linting

  Scenario: a dangling stable-id token in prose is flagged
    Given a Markdown file citing a non-existent REQ id
    When lint_docs is called on that file
    Then a W099 finding is returned with the file, line, and token

  Scenario: a resolvable reference is not flagged
    Given a Markdown file citing an existing element
    When lint_docs is called on that file
    Then no finding is returned for that reference
```
