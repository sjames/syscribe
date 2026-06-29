---
type: TestCase
id: TC-TRS-MCP-014
name: "explain_finding explains a validation code"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_authoring.rs
verifies:
  - REQ-TRS-MCP-014
tags:
  - mcp
---

```gherkin
Feature: Validation finding explanation

  Scenario: a known code is explained
    Given an initialized mcp server
    When explain_finding is called with a known validation code
    Then a human-readable explanation of the rule is returned
    And guidance on how to resolve it is included

  Scenario: an unknown code is rejected
    When explain_finding is called with a nonexistent code
    Then a structured error is returned
```
