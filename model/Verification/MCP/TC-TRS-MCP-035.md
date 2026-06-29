---
type: TestCase
id: TC-TRS-MCP-035
name: "Evidence/diagram tools honour the common structured CLI-parity contract"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_evidence.rs
verifies:
  - REQ-TRS-MCP-034
tags:
  - mcp
---

```gherkin
Feature: Common tool contract

  Scenario: read tools return structured JSON and accept any ref form
    Given an initialized mcp server over a fixture model
    When coverage_matrix is called
    Then the result is a structured object, not plain text

  Scenario: the write tool is dry-run by default
    When ingest_results is called without dry_run
    Then nothing is written and the response describes the would-be change
```
