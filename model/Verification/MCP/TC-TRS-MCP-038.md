---
type: TestCase
id: TC-TRS-MCP-038
name: "coverage_gaps returns the actionable, classified coverage subset"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_evidence.rs
verifies:
  - REQ-TRS-MCP-037
tags:
  - mcp
---

```gherkin
Feature: Coverage gaps

  Scenario: an uncovered approved requirement is reported
    Given a fixture with an approved requirement that has no verifying test case
    When coverage_gaps is called
    Then that requirement appears with gap class "uncovered" and its governing finding code
    And each row carries the requirement ref and gap class
```
