---
type: TestCase
id: TC-TRS-MCP-013
name: "template returns a frontmatter skeleton for a type"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_authoring.rs
verifies:
  - REQ-TRS-MCP-013
tags:
  - mcp
---

```gherkin
Feature: Element template

  Scenario: template returns a skeleton with required fields
    Given an initialized mcp server
    When template is called for the Requirement type
    Then the returned skeleton contains a type field set to Requirement
    And it contains the required fields as placeholders

  Scenario: an unknown type is rejected
    When template is called for a nonexistent type
    Then a structured error is returned
```
