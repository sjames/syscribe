---
type: TestCase
id: TC-TRS-MCP-004
name: "Spec sections are MCP resources and authoring prompts are MCP prompts"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_resources.rs
verifies:
  - REQ-TRS-MCP-004
tags:
  - mcp
---

Verifies that the embedded format spec is exposed as MCP resources and the create-model
authoring prompts as MCP prompts.

```gherkin
Feature: Spec resources and authoring prompts

  Scenario: spec sections are listed and readable as resources
    Given an initialized mcp server
    When a resources/list request is sent
    Then at least one resource with a syscribe://spec/ URI is returned
    And a resources/read of that URI returns the spec section text

  Scenario: authoring prompts are exposed
    When a prompts/list request is sent
    Then a create-model prompt is listed
    And a prompts/get for it returns the authoring instructions
```
