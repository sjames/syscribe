---
type: TestCase
id: TC-TRS-MCP-034
name: "run_report runs allowlisted reports and refuses disallowed commands and model redirection"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_report.rs
verifies:
  - REQ-TRS-MCP-033
tags:
  - mcp
  - security
---

```gherkin
Feature: Guarded report passthrough

  Scenario: an allowlisted report runs and returns its output
    Given an initialized mcp server over a fixture model
    When run_report is called with command "matrix"
    Then exitCode is 0
    And the report output is returned as non-empty text

  Scenario: a non-allowlisted command is refused
    When run_report is called with command "move"
    Then the call returns a tool error and nothing is executed

  Scenario: redirecting the model root is refused
    When run_report is called with command "matrix" and args that set -m to another path
    Then the call returns a tool error
```
