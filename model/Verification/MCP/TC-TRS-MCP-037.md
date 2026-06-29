---
type: TestCase
id: TC-TRS-MCP-037
name: "coverage_matrix returns the coverage grid and rollup with a passing/covered split"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_evidence.rs
verifies:
  - REQ-TRS-MCP-036
tags:
  - mcp
---

```gherkin
Feature: Coverage matrix

  Scenario: the grid and rollup are returned
    Given an initialized mcp server over a fixture with a feature model
    When coverage_matrix is called
    Then the result has columns, rows with per-cell states, and a coverage rollup

  Scenario: ingested results upgrade a covered cell to passing
    Given results have been committed marking the verifying test passed
    When coverage_matrix is called
    Then the requirement's cell state is passing
```
