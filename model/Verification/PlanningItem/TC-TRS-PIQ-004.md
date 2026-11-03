---
type: TestCase
id: TC-TRS-PIQ-004
name: "MCP questions and question tools"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_planning.rs
verifies:
  - REQ-TRS-PIQ-004
tags:
  - planning
---

```gherkin
Feature: MCP questions

  Scenario: read and write
    Then questions lists them, question dry-runs by default and commits with dry_run false

  Scenario: read-only server
    Then the question tool is not offered
```
