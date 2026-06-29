---
type: TestCase
id: TC-TRS-MCP-003
name: "Read tools return token-efficient, chainable structured results"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_read.rs
verifies:
  - REQ-TRS-MCP-003
tags:
  - mcp
---

Verifies the token-efficiency contract of the read tools: summary-by-default, opt-in detail,
bounded lists, flexible references, and chainable identifiers.

```gherkin
Feature: Token-efficient read tools

  Scenario: get_element returns a compact summary by default
    Given an initialized mcp server over a fixture model
    When get_element is called with a requirement reference and no detail flag
    Then the result carries qname and id
    And the result omits the full documentation body

  Scenario: get_element detail returns full fields on request
    When get_element is called with detail=true
    Then the result includes the full frontmatter fields and documentation body

  Scenario: search is bounded and reports a total
    When search is called with limit=2 over a query matching more than two elements
    Then at most two results are returned
    And the total count of matches is reported
    And every result carries qname and id

  Scenario: an element reference may be an id, a qualified name, or a display name
    When get_element is called with the same element's stable id and then its qualified name
    Then both calls resolve to the same element

  Scenario: graph_query traverses the model graph
    When graph_query is called from a requirement following verifies edges
    Then the returned nodes include the verifying test cases

  Scenario: validate returns whole-model findings and validate_element is scoped
    When validate is called with no element reference
    Then a structured findings list for the whole model is returned
    When validate_element is called with a single element reference
    Then a structured findings list scoped to that element is returned

  Scenario: trace returns the verification slice for a requirement
    When trace is called with a requirement reference
    Then the verifiedBy list contains the verifying test case

  Scenario: impact reports the elements that depend on an element
    When impact is called with a part definition reference
    Then the affected list is non-empty
    And it contains the subtype that depends on it via supertype
    And every affected entry carries a qname and a numeric distance
```
