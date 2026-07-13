---
type: TestCase
id: TC-TRS-MCP-047
name: "MCP baseline_list / baseline_diff / baseline_verify are read-only and correct"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_baseline.rs
tags:
  - mcp
  - baseline
verifies:
  - REQ-TRS-MCP-046
---

Verifies the MCP baseline read tools against a git-backed fixture with sealed baselines:
`baseline_list` inventories them, `baseline_diff` reports the changed element, `baseline_verify`
reports pass/fail, no write tool `baseline_create` is offered, and none mutate the model.

```gherkin
Feature: MCP baseline read tools

  Scenario: baseline_list inventories sealed baselines
    Given a model with two sealed baselines
    When baseline_list is called
    Then both baselines are reported with id, aggregateHash, and scope

  Scenario: baseline_diff reports the changed element
    Given two baselines differing by one element
    When baseline_diff is called with the two ids
    Then that element is reported under `changed`, keyed by stable id

  Scenario: baseline_verify reports integrity
    Given an intact baseline and a drifted one
    When baseline_verify is called
    Then the intact baseline has passed=true and the drifted one passed=false

  Scenario: No baseline_create write tool is exposed
    When the tool list is requested
    Then no `baseline_create` tool is present

  Scenario: The tools are read-only
    When the baseline read tools are called
    Then no model file is modified
```
