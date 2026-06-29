---
type: TestCase
id: TC-TRS-MCP-020
name: "list_by_type, tree, neighbors, and the get_element fields projection work"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_read.rs
verifies:
  - REQ-TRS-MCP-003
tags:
  - mcp
---

Additional coverage for read tools named by REQ-TRS-MCP-003 that the initial suite exercised
only indirectly.

```gherkin
Feature: Remaining read-tool coverage

  Scenario: list_by_type enumerates elements of a type with a total
    Given an initialized mcp server over a fixture model
    When list_by_type is called for the Requirement type
    Then the items include REQ-FX-001 and a total is reported

  Scenario: tree returns a containment subtree
    When tree is called at the model root
    Then a nested structure of qnames is returned

  Scenario: neighbors returns one-hop adjacencies
    When neighbors is called for REQ-FX-001
    Then the inbound adjacencies include the verifying test case

  Scenario: the fields projection limits returned keys
    When get_element is called with detail and a fields projection of a single field
    Then only the projected field (plus identity keys) is returned
```
