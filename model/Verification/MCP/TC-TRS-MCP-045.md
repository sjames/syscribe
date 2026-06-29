---
type: TestCase
id: TC-TRS-MCP-045
name: "coverage and coverage_matrix agree; draft-only-linked requirements are planned"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_evidence.rs
verifies:
  - REQ-TRS-MCP-044
tags:
  - mcp
---

```gherkin
Feature: Unified coverage classifier

  Scenario: a draft-only-linked requirement is planned, not verified
    Given a fixture where REQ-FXPLAN-001 is verified only by a draft TestCase
    When coverage is called
    Then REQ-FXPLAN-001 is in the planned set, not the verified set
    When coverage_matrix is called
    Then the REQ-FXPLAN-001 cell is "planned"

  Scenario: no cross-surface contradiction (Invariant 2)
    Given a generated coverage and coverage_matrix over the fixture
    Then no requirement counted verified by coverage is a gap in every configuration of the matrix

  Scenario: a non-draft-linked requirement is verified and covered
    Given REQ-FX-001 is verified by a non-draft TestCase
    Then coverage counts it verified
    And its coverage_matrix cell is "covered" or "passing", never "gap"
```
