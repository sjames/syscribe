---
type: TestCase
id: TC-TRS-MCP-019
name: "coverage tool summarises requirement verification coverage"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_authoring.rs
verifies:
  - REQ-TRS-MCP-019
tags:
  - mcp
---

```gherkin
Feature: Verification coverage summary

  Scenario: coverage reports verified and unverified requirements
    Given an initialized mcp server over a fixture with one verified and one unverified requirement
    When the coverage tool is called
    Then the verified requirement count is at least one
    And the unverified list contains the unverified requirement's id
    And each listed requirement carries its qname and id
```
