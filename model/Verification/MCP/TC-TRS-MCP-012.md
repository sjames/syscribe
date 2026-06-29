---
type: TestCase
id: TC-TRS-MCP-012
name: "describe_type returns a type's structured frontmatter schema"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_authoring.rs
verifies:
  - REQ-TRS-MCP-012
tags:
  - mcp
---

```gherkin
Feature: Type schema introspection

  Scenario: describe_type reports fields and enum domains
    Given an initialized mcp server
    When describe_type is called for the Requirement type
    Then the result lists frontmatter fields with required flags and value types
    And the status field reports its permitted enumerated values
    And the result is structured JSON, not prose
```
