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

  Scenario: coverage partitions leaf gaps from parents missing an integration test
    Given an initialized mcp server over a fixture with verified, unverified-leaf, and parent requirements
    When the coverage tool is called
    Then the verified requirement count is at least one
    And unverifiedLeaves contains the approved leaf requirements with no TestCase
    And parentsMissingIntegrationTest contains an approved parent that has only a unit-level (L1/L2) test
    And a draft requirement with no TestCase is excluded from both gap lists (planned, not a gap)
    And a parent is never listed as a leaf gap, nor a leaf as a parent gap
    And each listed requirement carries its qname and id, and parents carry a child count
```
