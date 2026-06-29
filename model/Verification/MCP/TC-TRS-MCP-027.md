---
type: TestCase
id: TC-TRS-MCP-027
name: "search filters by type, matches body text, and honours a where predicate"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_retrieval.rs
verifies:
  - REQ-TRS-MCP-026
tags:
  - mcp
  - retrieval
---

```gherkin
Feature: Filtered search

  Scenario: a type filter narrows results
    Given an initialized mcp server over a fixture model
    When search is called with query "fixture" and type "Requirement"
    Then every result has type Requirement
    And REQ-FX-001 is among them

  Scenario: full-text body matching
    When search is called with a term that appears only in an element's body
    Then the element whose body contains the term is returned

  Scenario: a custom-field where predicate filters results
    When search is called with where custom.customKey=keepme
    Then the result set contains the element carrying that custom field
```
