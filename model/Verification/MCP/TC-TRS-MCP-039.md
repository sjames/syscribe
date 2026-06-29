---
type: TestCase
id: TC-TRS-MCP-039
name: "evidence returns a requirement's verification chain with verdicts"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_evidence.rs
verifies:
  - REQ-TRS-MCP-038
tags:
  - mcp
---

```gherkin
Feature: Evidence drill-down

  Scenario: the chain reports unknown verdict with no results
    Given an initialized mcp server with no results sidecar
    When evidence is called for REQ-FX-001
    Then it lists TC-FX-001 with its test function and a verdict of unknown

  Scenario: the chain reports a pass after ingestion
    Given results have been committed marking the test function passed
    When evidence is called for REQ-FX-001
    Then the verdict for that test function is pass
```
